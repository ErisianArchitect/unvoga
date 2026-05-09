use rollgrid::rollgrid2d::Bounds2D;
use unvoga::blockstate;
use unvoga::core::voxel::procgen::worldgenerator::WorldGenerator;
use unvoga::core::voxel::world::{VoxelWorld, WORLD_BOTTOM, WORLD_TOP};
use unvoga::prelude::Id;

const COURSE_BASE_Y: f64 = 0.0;
const SOIL_DEPTH: i32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CourseSurface {
    Tee,
    Green,
    Fairway,
    FirstCut,
    Rough,
    Bunker,
}

#[derive(Debug, Clone, Copy)]
struct Point2 {
    x: f64,
    z: f64,
}

impl Point2 {
    const fn new(x: f64, z: f64) -> Self {
        Self { x, z }
    }

    fn distance(self, other: Self) -> f64 {
        let dx = self.x - other.x;
        let dz = self.z - other.z;
        (dx * dx + dz * dz).sqrt()
    }
}

#[derive(Debug, Clone)]
struct Hole {
    path: Vec<Point2>,
    green: Point2,
    fairway_width: f64,
    green_radius: f64,
}

#[derive(Debug, Clone, Copy)]
struct Bunker {
    center: Point2,
    radius_x: f64,
    radius_z: f64,
}

#[derive(Debug, Clone, Copy)]
struct CourseBlocks {
    stone: Id,
    dirt: Id,
    tee: Id,
    fairway: Id,
    first_cut: Id,
    green: Id,
    rough: Id,
    bunker: Id,
}

pub struct GolfCourseGenerator {
    holes: Vec<Hole>,
    bunkers: Vec<Bunker>,
    blocks: CourseBlocks,
}

impl GolfCourseGenerator {
    pub fn new() -> Self {
        Self::with_blocks(CourseBlocks {
            stone: blockstate!(stone).register(),
            dirt: blockstate!(dirt).register(),
            tee: blockstate!(tee).register(),
            fairway: blockstate!(fairway).register(),
            first_cut: blockstate!(first_cut).register(),
            green: blockstate!(green).register(),
            rough: blockstate!(rough).register(),
            bunker: blockstate!(bunker).register(),
        })
    }

    fn with_blocks(blocks: CourseBlocks) -> Self {
        Self {
            holes: vec![
                Hole {
                    path: vec![
                        Point2::new(0.0, -18.0),
                        Point2::new(10.0, 35.0),
                        Point2::new(-8.0, 82.0),
                        Point2::new(-8.0, 104.0),
                    ],
                    green: Point2::new(-8.0, 104.0),
                    fairway_width: 13.0,
                    green_radius: 15.0,
                },
                Hole {
                    path: vec![
                        Point2::new(42.0, 112.0),
                        Point2::new(76.0, 160.0),
                        Point2::new(52.0, 220.0),
                        Point2::new(50.0, 245.0),
                    ],
                    green: Point2::new(50.0, 245.0),
                    fairway_width: 12.0,
                    green_radius: 14.0,
                },
                Hole {
                    path: vec![
                        Point2::new(-58.0, 92.0),
                        Point2::new(-86.0, 142.0),
                        Point2::new(-62.0, 190.0),
                        Point2::new(-92.0, 234.0),
                    ],
                    green: Point2::new(-92.0, 234.0),
                    fairway_width: 11.0,
                    green_radius: 13.0,
                },
            ],
            bunkers: vec![
                Bunker { center: Point2::new(-21.0, 96.0), radius_x: 7.0, radius_z: 4.5 },
                Bunker { center: Point2::new(7.0, 111.0), radius_x: 5.0, radius_z: 7.0 },
                Bunker { center: Point2::new(36.0, 236.0), radius_x: 6.5, radius_z: 5.0 },
                Bunker { center: Point2::new(63.0, 252.0), radius_x: 5.5, radius_z: 6.0 },
                Bunker { center: Point2::new(-80.0, 224.0), radius_x: 6.0, radius_z: 4.5 },
            ],
            blocks,
        }
    }

    fn column(&self, x: i32, z: i32) -> (i32, CourseSurface) {
        let point = Point2::new(x as f64, z as f64);
        let surface = self.surface_at(point);
        let height = self.height_at(point, surface);
        (height.round().clamp((WORLD_BOTTOM + 1) as f64, (WORLD_TOP - 1) as f64) as i32, surface)
    }

    fn surface_at(&self, point: Point2) -> CourseSurface {
        let mut best = CourseSurface::Rough;
        let mut best_priority = 0;

        for hole in &self.holes {
            let tee = hole.path[0];
            if ellipse(point, tee, 10.0, 7.0) <= 1.0 {
                set_surface(&mut best, &mut best_priority, CourseSurface::Tee);
            }

            let green_distance = point.distance(hole.green);
            if green_distance <= hole.green_radius {
                set_surface(&mut best, &mut best_priority, CourseSurface::Green);
            }

            let distance_to_path = distance_to_polyline(point, &hole.path).0;
            if distance_to_path <= hole.fairway_width {
                set_surface(&mut best, &mut best_priority, CourseSurface::Fairway);
            } else if distance_to_path <= hole.fairway_width + 7.0 {
                set_surface(&mut best, &mut best_priority, CourseSurface::FirstCut);
            }
        }

        for bunker in &self.bunkers {
            if ellipse(point, bunker.center, bunker.radius_x, bunker.radius_z) <= 1.0 {
                set_surface(&mut best, &mut best_priority, CourseSurface::Bunker);
            }
        }

        best
    }

    fn height_at(&self, point: Point2, surface: CourseSurface) -> f64 {
        let rough = COURSE_BASE_Y
            + 1.4 * (point.x * 0.045).sin()
            + 0.9 * (point.z * 0.037).cos()
            + 0.45 * ((point.x + point.z) * 0.026).sin();

        let (course_distance, course_t) = self.closest_course_path(point);
        let course_grade = COURSE_BASE_Y + course_t * 5.0;
        let fairway_blend = smoothstep(24.0, 0.0, course_distance);
        let shaped = rough * (1.0 - fairway_blend) + course_grade * fairway_blend;

        match surface {
            CourseSurface::Tee => COURSE_BASE_Y,
            CourseSurface::Green => {
                let green_y = self.closest_green_height(point);
                shaped * 0.15 + green_y * 0.85
            }
            CourseSurface::Bunker => shaped - 1.0,
            _ => shaped,
        }
    }

    fn closest_course_path(&self, point: Point2) -> (f64, f64) {
        self.holes
            .iter()
            .map(|hole| distance_to_polyline(point, &hole.path))
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .unwrap_or((f64::MAX, 0.0))
    }

    fn closest_green_height(&self, point: Point2) -> f64 {
        self.holes
            .iter()
            .min_by(|a, b| {
                point
                    .distance(a.green)
                    .partial_cmp(&point.distance(b.green))
                    .unwrap()
            })
            .map(|hole| {
                let (_, t) = distance_to_polyline(hole.green, &hole.path);
                COURSE_BASE_Y + t * 5.0
            })
            .unwrap_or(COURSE_BASE_Y)
    }

    fn block_for(&self, y: i32, top: i32, surface: CourseSurface) -> Id {
        if y == top - 1 {
            match surface {
                CourseSurface::Tee => self.blocks.tee,
                CourseSurface::Green => self.blocks.green,
                CourseSurface::Fairway => self.blocks.fairway,
                CourseSurface::FirstCut => self.blocks.first_cut,
                CourseSurface::Rough => self.blocks.rough,
                CourseSurface::Bunker => self.blocks.bunker,
            }
        } else if y >= top - SOIL_DEPTH {
            self.blocks.dirt
        } else {
            self.blocks.stone
        }
    }
}

impl Default for GolfCourseGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldGenerator for GolfCourseGenerator {
    fn generate_chunk(&mut self, world: &mut VoxelWorld, area: Bounds2D) {
        for (x, z) in area.iter() {
            let (top, surface) = self.column(x, z);
            for y in WORLD_BOTTOM..top {
                world.set_block((x, y, z), self.block_for(y, top, surface));
            }
        }
    }
}

fn set_surface(best: &mut CourseSurface, best_priority: &mut u8, surface: CourseSurface) {
    let priority = surface_priority(surface);
    if priority > *best_priority {
        *best = surface;
        *best_priority = priority;
    }
}

fn surface_priority(surface: CourseSurface) -> u8 {
    match surface {
        CourseSurface::Rough => 0,
        CourseSurface::FirstCut => 1,
        CourseSurface::Fairway => 2,
        CourseSurface::Tee => 3,
        CourseSurface::Green => 4,
        CourseSurface::Bunker => 5,
    }
}

fn ellipse(point: Point2, center: Point2, radius_x: f64, radius_z: f64) -> f64 {
    let dx = (point.x - center.x) / radius_x;
    let dz = (point.z - center.z) / radius_z;
    dx * dx + dz * dz
}

fn distance_to_polyline(point: Point2, path: &[Point2]) -> (f64, f64) {
    let mut best_distance = f64::MAX;
    let mut best_progress = 0.0;
    let segment_count = path.len().saturating_sub(1);

    for (index, segment) in path.windows(2).enumerate() {
        let (distance, local_t) = distance_to_segment(point, segment[0], segment[1]);
        if distance < best_distance {
            best_distance = distance;
            best_progress = (index as f64 + local_t) / segment_count.max(1) as f64;
        }
    }

    (best_distance, best_progress)
}

fn distance_to_segment(point: Point2, a: Point2, b: Point2) -> (f64, f64) {
    let ab_x = b.x - a.x;
    let ab_z = b.z - a.z;
    let ap_x = point.x - a.x;
    let ap_z = point.z - a.z;
    let ab_len_sq = ab_x * ab_x + ab_z * ab_z;
    if ab_len_sq <= f64::EPSILON {
        return (point.distance(a), 0.0);
    }

    let t = ((ap_x * ab_x + ap_z * ab_z) / ab_len_sq).clamp(0.0, 1.0);
    let closest = Point2::new(a.x + ab_x * t, a.z + ab_z * t);
    (point.distance(closest), t)
}

fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_generator() -> GolfCourseGenerator {
        GolfCourseGenerator::with_blocks(CourseBlocks {
            stone: Id::AIR,
            dirt: Id::AIR,
            tee: Id::AIR,
            fairway: Id::AIR,
            first_cut: Id::AIR,
            green: Id::AIR,
            rough: Id::AIR,
            bunker: Id::AIR,
        })
    }

    #[test]
    fn classifies_course_surfaces() {
        let generator = test_generator();

        assert_eq!(generator.surface_at(Point2::new(0.0, -18.0)), CourseSurface::Tee);
        assert_eq!(generator.surface_at(Point2::new(8.0, 35.0)), CourseSurface::Fairway);
        assert_eq!(generator.surface_at(Point2::new(-8.0, 104.0)), CourseSurface::Green);
        assert_eq!(generator.surface_at(Point2::new(-21.0, 96.0)), CourseSurface::Bunker);
        assert_eq!(generator.surface_at(Point2::new(120.0, -80.0)), CourseSurface::Rough);
    }

    #[test]
    fn course_columns_stay_inside_world_bounds() {
        let generator = test_generator();
        let (top, _) = generator.column(0, 0);

        assert!(top > WORLD_BOTTOM);
        assert!(top < WORLD_TOP);
    }
}
