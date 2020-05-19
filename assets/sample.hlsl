static const uint GROUP_X = 8;
static const uint GROUP_Y = 32;
static const uint NUM_THREADS = GROUP_X * GROUP_Y;
static const uint NUM_WAVES = NUM_THREADS / 32;

static const uint PRIMITIVE_LINE = 1;

RWTexture2D<float4> render_target : register(u0, space0);

struct Locals {
    uint2 num_tiles;
    float2 viewport_offset;
    float2 viewport_extent;
    uint num_objects;
};
ConstantBuffer<Locals> u_locals : register(b0, space1);

struct Object {
    uint2 primitives;
    uint offset_data;
    float4 bbox;
};
StructuredBuffer<Object> t_objects : register(t0, space1);
Buffer<uint> t_primitives : register(t1, space1);
Buffer<uint4> t_data : register(t2, space1);

float line_eval(float p0, float p1, float t) {
    return lerp(p0, p1, t);
}

float line_raycast(float p0, float p1, float p) {
   return (p - p0) / (p1 - p0);
}

float cdf(float x, float slope) {
    return saturate(x * slope + 0.5);
}

struct ObjectData {
    uint2 primitives;
    uint offset_data;
};
groupshared ObjectData local_objects[GROUP_X][GROUP_Y];

struct Intersection {
    float distance;
    float slope;
    float dx;
    float min_y;
};
groupshared Intersection line_intersect[GROUP_X][GROUP_Y];

[numthreads(GROUP_X * GROUP_Y, 1, 1)]
void test(
    uint3 invocation_id: SV_GroupThreadID,
    uint3 group_id: SV_GroupID
) {
    const uint lane = WaveGetLaneIndex();
    const uint2 group_thread_id = uint2(invocation_id.x / WaveGetLaneCount(), lane);
    const float2 tile_goup_extent = u_locals.viewport_extent / u_locals.num_tiles;
    const float2 tile_group_offset = u_locals.viewport_offset + tile_goup_extent * group_id.xy;

    const float2 dxdy = tile_goup_extent / uint2(GROUP_X, GROUP_Y);
    const float2 unit = 1.0 / dxdy;

    const float2 wave_start = tile_group_offset + float2(group_thread_id.x + 0.5, 0.0) * dxdy;

    float coverage = 0.0;

    for (uint i = 0; i < u_locals.num_objects; i += GROUP_Y) {
        bool intersection = false;
        Object object;
        if (i + lane < u_locals.num_objects) {
            object = t_objects[i + lane];
            const float4 bbox = object.bbox - float4(tile_group_offset.x, tile_group_offset.y, tile_group_offset.x, tile_group_offset.y);
            intersection = (bbox.z >= 0.0)
                && (bbox.w >= 0.0)
                && (tile_goup_extent.x >= bbox.x)
                && (tile_goup_extent.y >= bbox.y);
        }

        uint offset = WavePrefixCountBits(intersection);

        if (intersection) {
            local_objects[group_thread_id.x][offset].primitives = object.primitives;
            local_objects[group_thread_id.x][offset].offset_data = object.offset_data;
        }

        const uint num_intersections = WaveActiveCountBits(intersection);
        for (uint o = 0; o < num_intersections; o++) {
            const ObjectData local_obj = local_objects[group_thread_id.x][o];
            float local_coverage = 0.0;

            uint base = local_obj.offset_data;
            for (uint p = local_obj.primitives.x; p < local_obj.primitives.y; p += 32) {
                if (p + lane < local_obj.primitives.y) {
                    const uint vertex_offset = local_obj.offset_data + (p - local_obj.primitives.x + lane);
                    uint4 vertices = t_data[vertex_offset];
                    const float2 p0 = asfloat(vertices.xy) - wave_start;
                    const float2 p1 = asfloat(vertices.zw) - wave_start;

                    Intersection line_intersection;
                    line_intersection.min_y = min(p0.y, p1.y);
                    line_intersection.dx = 0.0;

                    const float max_y = max(p0.y, p1.y);
                    if (max_y >= 0.0) {
                        const float xx0 = clamp(p0.x, -0.5 * dxdy.x, 0.5 * dxdy.x);
                        const float xx1 = clamp(p1.x, -0.5 * dxdy.x, 0.5 * dxdy.x);
                        line_intersection.dx = (xx1 - xx0) * unit.x;

                        const float t = line_raycast(p0.x, p1.x, 0.5 * (xx0 + xx1)); // raycast y direction at sample pos
                        const float d = line_eval(p0.y, p1.y, t) * unit.y; // get x value at ray intersection
                        const float2 tangent = abs(p1 - p0);
                        const float m = tangent.x / max(tangent.x, tangent.y);
                        line_intersection.distance = d;
                        line_intersection.slope = m;
                    }

                    line_intersect[group_thread_id.x][group_thread_id.y] = line_intersection;
                }

                const uint num_lanes = WaveActiveCountBits(p + lane < local_obj.primitives.y);
                for (uint l = 0; l < num_lanes; l++) {
                    Intersection local_intersection = line_intersect[group_thread_id.x][l];
                    float cy = 1.0;
                    if (local_intersection.min_y < ((group_thread_id.y + 1) * dxdy.y)) {
                        cy = cdf(local_intersection.distance - (group_thread_id.y + 0.5), local_intersection.slope);
                    }
                    local_coverage += cy * local_intersection.dx;
                }
            }

            coverage += saturate(local_coverage);
        }
    }

    float color = saturate(coverage / 6.0);
    const uint2 thread_id = group_id.xy * uint2(GROUP_X, GROUP_Y) + group_thread_id;
    render_target[thread_id.xy] = float4(color, color, color, 1.0);
}
