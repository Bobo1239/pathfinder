// pathfinder/path-utils/src/curve.rs
//
// Copyright © 2017 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Geometry utilities for Bézier curves.

use euclid::approxeq::ApproxEq;
use euclid::Point2D;
use std::f32;

use PathCommand;
use intersection::Intersect;
use line::Line;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Curve {
    pub endpoints: [Point2D<f32>; 2],
    pub control_point: Point2D<f32>,
}

impl Curve {
    #[inline]
    pub fn new(endpoint_0: &Point2D<f32>, control_point: &Point2D<f32>, endpoint_1: &Point2D<f32>)
               -> Curve {
        Curve {
            endpoints: [*endpoint_0, *endpoint_1],
            control_point: *control_point,
        }
    }

    #[inline]
    pub fn sample(&self, t: f32) -> Point2D<f32> {
        let (p0, p1, p2) = (&self.endpoints[0], &self.control_point, &self.endpoints[1]);
        Point2D::lerp(&p0.lerp(*p1, t), p1.lerp(*p2, t), t)
    }

    #[inline]
    pub fn subdivide(&self, t: f32) -> (Curve, Curve) {
        let (p0, p1, p2) = (&self.endpoints[0], &self.control_point, &self.endpoints[1]);
        let (ap1, bp1) = (p0.lerp(*p1, t), p1.lerp(*p2, t));
        let ap2bp0 = ap1.lerp(bp1, t);
        (Curve::new(p0, &ap1, &ap2bp0), Curve::new(&ap2bp0, &bp1, p2))
    }

    pub fn subdivide_at_x(&self, x: f32) -> (Curve, Curve) {
        let (prev_part, next_part) = self.subdivide(self.solve_t_for_x(x));
        if self.endpoints[0].x <= self.endpoints[1].x {
            (prev_part, next_part)
        } else {
            (next_part, prev_part)
        }
    }

    #[inline]
    pub fn to_path_segment(&self) -> PathCommand {
        PathCommand::CurveTo(self.control_point, self.endpoints[1])
    }

    pub fn inflection_points(&self) -> (Option<f32>, Option<f32>) {
        let inflection_point_x = Curve::inflection_point_x(self.endpoints[0].x,
                                                           self.control_point.x,
                                                           self.endpoints[1].x);
        let inflection_point_y = Curve::inflection_point_x(self.endpoints[0].y,
                                                           self.control_point.y,
                                                           self.endpoints[1].y);
        (inflection_point_x, inflection_point_y)
    }

    /// Uses the Citardauq Formula to avoid precision problems.
    ///
    /// https://math.stackexchange.com/a/311397
    pub fn solve_t_for_x(&self, x: f32) -> f32 {
        let p0x = self.endpoints[0].x as f64;
        let p1x = self.control_point.x as f64;
        let p2x = self.endpoints[1].x as f64;
        let x = x as f64;

        let a = p0x - 2.0 * p1x + p2x;
        let b = -2.0 * p0x + 2.0 * p1x;
        let c = p0x - x;

        let t = 2.0 * c / (-b - (b * b - 4.0 * a * c).sqrt());
        t.max(0.0).min(1.0) as f32
    }

    #[inline]
    pub fn solve_y_for_x(&self, x: f32) -> f32 {
        self.sample(self.solve_t_for_x(x)).y
    }

    #[inline]
    pub fn baseline(&self) -> Line {
        Line::new(&self.endpoints[0], &self.endpoints[1])
    }

    #[inline]
    fn inflection_point_x(endpoint_x_0: f32, control_point_x: f32, endpoint_x_1: f32)
                          -> Option<f32> {
        let num = endpoint_x_0 - control_point_x;
        let denom = endpoint_x_0 - 2.0 * control_point_x + endpoint_x_1;
        let t = num / denom;
        if t > f32::approx_epsilon() && t < 1.0 - f32::approx_epsilon() {
            Some(t)
        } else {
            None
        }
    }

    #[inline]
    pub fn intersect<T>(&self, other: &T) -> Option<Point2D<f32>> where T: Intersect {
        <Curve as Intersect>::intersect(self, other)
    }
}
