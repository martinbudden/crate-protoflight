#![allow(unused)]
use core::f32::consts::{FRAC_1_SQRT_2, FRAC_PI_2};
use vqm::{TrigonometricMethods, Vector2df32};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PathFollower {
    lateral_accel: f32,    // Lateral acceleration setpoint in m/s^2
    l1_distance: f32,      // L1 lead distance, defined by period and damping
    nav_bearing: f32,      // bearing to L1 reference point
    crosstrack_error: f32, // crosstrack error in meters
    target_bearing: f32,   // the heading setpoint
    l1_ratio: f32,         // L1 ratio for navigation
    k_l1: f32,             // L1 control gain for _L1_damping
}

impl PathFollower {
    pub const fn new() -> Self {
        Self {
            lateral_accel: 0.0,
            l1_distance: 20.0,
            nav_bearing: 0.0,
            crosstrack_error: 0.0,
            target_bearing: 0.0,
            l1_ratio: 5.0,
            k_l1: 2.0,
        }
    }
}

impl Default for PathFollower {
    fn default() -> Self {
        Self::new()
    }
}

impl PathFollower {
    // allow non snake_case so we can use the convention that AB means the vector from A to B.
    #![allow(non_snake_case)]
    pub fn navigate_waypoints(&mut self, A: Vector2df32, B: Vector2df32, P: Vector2df32, ground_velocity: Vector2df32) {
        // this follows the logic presented in [1]
        let mut eta;

        // get the direction between the last (visited) and next waypoint
        let PB_normalized = (B - P).normalize();
        self.target_bearing = (PB_normalized.y).atan2(PB_normalized.x);

        // enforce a minimum ground speed of 0.1 m/s to avoid singularities
        let ground_speed = ground_velocity.norm().max(0.1);

        // calculate the L1 length required for the desired period
        self.l1_distance = self.l1_ratio * ground_speed;

        // calculate vector from A to B
        let mut AB = B - A;
        // check if waypoints are on top of each other.
        // If yes, skip A and directly continue to B
        if AB.norm_squared() < 1.0e-3 {
            AB = B - P;
        }
        AB = AB.normalize();

        // calculate the vector from waypoint A to the aircraft (P)
        let AP = P - A;

        // calculate crosstrack error (output only)
        self.crosstrack_error = AB.cross(AP);

        // If the current position is in a +-135 degree angle behind waypoint A
        // and further away from A than the L1 distance, then A becomes the L1 point.
        // If the aircraft is already between A and B normal L1 logic is applied.

        // estimate aircraft position WRT to B
        let BP_normalized = (P - B).normalize();

        // calculate angle of aircraft position vector relative to line
        let AB_BP_bearing = BP_normalized.cross(AB).atan2(BP_normalized.dot(AB));

        // extension from [2], fly directly to A
        let distance_AP = AP.norm();
        let along_track_distance = AP.dot(B);
        if (distance_AP > self.l1_distance) && (along_track_distance / distance_AP.max(1.0) < -FRAC_1_SQRT_2) {
            // calculate eta to fly to waypoint A

            // unit vector from waypoint A to current position
            let AP_normalized = AP.normalize();

            // velocity across / orthogonal to line
            let cross_track_velocity = ground_velocity.cross(-AP_normalized);

            // velocity along line
            let along_track_velocity = ground_velocity.dot(-AP_normalized);
            eta = cross_track_velocity.atan2(along_track_velocity);

            // bearing from current position to L1 point
            self.nav_bearing = (-AP_normalized.y).atan2(-AP_normalized.x);

            // If the AB vector and the vector from B to aircraft point in the same
            // direction, we have missed the waypoint. At +- 90 degrees we are just passing it.
        } else if AB_BP_bearing.abs() < 1.5 {
            //math::radians(100.0F)) {
            // Extension, fly back to waypoint.
            // This corner case is possible if the system was following the AB line from waypoint A to waypoint B,
            // and then is switched to manual mode (or otherwise misses the waypoint)
            // and from behind the waypoint continues to follow the AB line.

            // calculate eta to fly to waypoint B

            // velocity across / orthogonal to line
            let cross_track_velocity = ground_velocity.cross(-BP_normalized);

            // velocity along line
            let along_track_velocity = ground_velocity.dot(-BP_normalized);
            eta = cross_track_velocity.atan2(along_track_velocity);

            // bearing from current position to L1 point
            self.nav_bearing = (-BP_normalized.y).atan2(-BP_normalized.x);
        } else {
            // calculate eta to fly along the line between A and B

            // velocity across / orthogonal to line
            let cross_track_velocity = ground_velocity.cross(AB);

            // velocity along line
            let along_track_velocity = ground_velocity.dot(AB);

            // calculate eta2 (angle of velocity vector relative to line)
            let eta2 = cross_track_velocity.atan2(along_track_velocity);

            // calculate eta1 (angle to L1 point)
            let cross_track_error = AP.cross(AB);
            let sine_eta1 = (cross_track_error / (self.l1_distance).max(0.1)).clamp(-1.0, 1.0);
            let eta1 = (sine_eta1).asin();
            eta = eta1 + eta2;

            // bearing from current position to L1 point
            self.nav_bearing = (AB.y).atan2(AB.x) + eta1;
        }

        // limit angle to +-90 degrees
        eta = eta.clamp(-FRAC_PI_2, FRAC_PI_2);
        self.lateral_accel = self.k_l1 * ground_speed * ground_speed / self.l1_distance * eta.sin();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<PathFollower>();
    }
    #[test]
    #[allow(clippy::float_cmp)]
    fn test_new() {
        let path_follower = PathFollower::new();
        assert_eq!(0.0, path_follower.lateral_accel);
    }
}
