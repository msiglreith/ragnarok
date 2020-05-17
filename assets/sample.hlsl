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
Buffer<uint> t_data : register(t2, space1);

float line_eval(float p0, float p1, float t) {
    return lerp(p0, p1, t);
}

float line_raycast(float p0, float p1, float p) {
   return (p - p0) / (p1 - p0);
}

float cdf(float x, float slope) {
    return saturate(x * slope + 0.5);
}

[numthreads(GROUP_X, GROUP_Y, 1)]
void test(
    uint3 group_thread_id: SV_GroupThreadID,
    uint3 group_id: SV_GroupID,
    uint3 thread_id: SV_DispatchThreadID
) {
    const float2 tile_goup_extent = u_locals.viewport_extent / u_locals.num_tiles;
    const float2 tile_group_offset = u_locals.viewport_offset + tile_goup_extent * group_id.xy;

    const float2 dxdy = tile_goup_extent / uint2(GROUP_X, GROUP_Y);
    const float2 unit = 1.0 / dxdy;

    const float2 fragment_center = tile_group_offset + (group_thread_id.xy + float2(0.5, 0.5)) * dxdy;

    float coverage = 0.0;

    for (uint i = 0; i < u_locals.num_objects; i += 1) {
        const Object object = t_objects[i];
        const float4 bbox = object.bbox - float4(tile_group_offset.x, tile_group_offset.y, tile_group_offset.x, tile_group_offset.y);
        bool intersection = (bbox.z >= 0.0)
            && (bbox.w >= 0.0)
            && (tile_goup_extent.x >= bbox.x)
            && (tile_goup_extent.y >= bbox.y);

        if (!intersection) {
            continue;
        }

        float local_coverage = 0.0;

        uint base = object.offset_data;
        for (uint p = object.primitives.x; p < object.primitives.y; p++) {
            const float2 p0 = float2(asfloat(t_data[base]), asfloat(t_data[base+1])) - fragment_center;
            const float2 p1 = float2(asfloat(t_data[base+2]), asfloat(t_data[base+3])) - fragment_center;
            base += 4;

            if (max(p0.y, p1.y) >= -0.5 * dxdy.y) {
                const float xx0 = clamp(p0.x, -0.5 * dxdy.x, 0.5 * dxdy.x);
                const float xx1 = clamp(p1.x, -0.5 * dxdy.x, 0.5 * dxdy.x);
                const float xx = (xx1 - xx0) * unit.x;

                float cy = 1.0;
                if (xx != 0.0 && min(p0.y, p1.y) < 0.5 * dxdy.y) {
                    const float t = line_raycast(p0.x, p1.x, 0.5 * (xx0 + xx1)); // raycast y direction at sample pos
                    const float d = line_eval(p0.y, p1.y, t) * unit.y; // get x value at ray intersection
                    const float2 tangent = abs(p1 - p0);
                    const float m = tangent.x / max(tangent.x, tangent.y);
                    cy = cdf(d, m);
                }

                local_coverage += cy * xx;
            }
        }

        coverage += saturate(local_coverage);
    }

    float color = saturate(coverage / 5.0);
    render_target[thread_id.xy] = float4(color, color, color, 1.0);
}
