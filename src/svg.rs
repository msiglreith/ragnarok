use kurbo::Shape;
use pathbreaker::kurbo;
use std::error;
use std::path::Path;

const PRIMITIVE_LINE: u32 = 1;

#[repr(C)]
#[derive(Debug)]
pub struct Object {
    primitives: [u32; 2],
    offset_data: u32,
    bbox: [f32; 4],
}

pub struct GpuData {
    pub objects: Vec<Object>,
    pub primitives: Vec<u32>,
    pub data: Vec<f32>,
}

impl GpuData {
    pub fn new() -> Self {
        GpuData {
            objects: Vec::new(),
            primitives: Vec::new(),
            data: Vec::new(),
        }
    }
}

pub fn generate_gpu_data(paths: &[kurbo::BezPath]) -> GpuData {
    let mut gpu_data = GpuData::new();

    for path in paths {
        let aabb = path.bounding_box();

        let data_offset = gpu_data.data.len();
        let primitive_start = gpu_data.primitives.len();

        let mut first = kurbo::Point::ZERO;
        let mut last = kurbo::Point::ZERO;
        for elem in path {
            match elem {
                kurbo::PathEl::MoveTo(p) => {
                    first = p;
                    last = p;
                }
                kurbo::PathEl::LineTo(p) => {
                    gpu_data.primitives.push(PRIMITIVE_LINE);
                    // p0
                    gpu_data.data.push(last.x as f32);
                    gpu_data.data.push(last.y as f32);
                    // p1
                    gpu_data.data.push(p.x as f32);
                    gpu_data.data.push(p.y as f32);

                    last = p;
                }
                kurbo::PathEl::ClosePath => {
                    gpu_data.primitives.push(PRIMITIVE_LINE);
                    // p0
                    gpu_data.data.push(last.x as f32);
                    gpu_data.data.push(last.y as f32);
                    // p1
                    gpu_data.data.push(first.x as f32);
                    gpu_data.data.push(first.y as f32);

                    last = first;
                }
                _ => todo!(),
            }
        }

        let primitive_end = gpu_data.primitives.len();
        // println!("{:?}", primitive_end - primitive_start);
        gpu_data.objects.push(Object {
            primitives: [primitive_start as _, primitive_end as _],
            offset_data: data_offset as _,
            bbox: [aabb.x0 as _, aabb.y0 as _, aabb.x1 as _, aabb.y1 as _],
        });
    }

    gpu_data
}

pub fn parse_svg<P: AsRef<Path>>(path: P) -> Result<Vec<kurbo::BezPath>, Box<dyn error::Error>> {
    let mut paths = Vec::new();

    let tree = usvg::Tree::from_file(path, &usvg::Options::default())?;
    for child in tree.root().children() {
        match *child.borrow() {
            usvg::NodeKind::Path(ref p) => {
                let mut path = kurbo::BezPath::new();
                for segment in p.data.0.iter() {
                    match *segment {
                        usvg::PathSegment::MoveTo { x, y } => {
                            path.move_to(kurbo::Point::new(x, y));
                        }
                        usvg::PathSegment::LineTo { x, y } => {
                            path.line_to(kurbo::Point::new(x, y));
                        }
                        usvg::PathSegment::CurveTo {
                            x1,
                            y1,
                            x2,
                            y2,
                            x,
                            y,
                        } => {
                            path.curve_to(
                                kurbo::Point::new(x1, y1),
                                kurbo::Point::new(x2, y2),
                                kurbo::Point::new(x, y),
                            );
                        }
                        usvg::PathSegment::ClosePath => {
                            path.close_path();
                        }
                    }
                }
                if let Some(ref fill) = p.fill {
                    if let usvg::Paint::Color(_) = fill.paint {
                        let path =
                            pathbreaker::break_path(&path, pathbreaker::CubicApprox::Flatten(0.1));

                        paths.push(path);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(paths)
}
