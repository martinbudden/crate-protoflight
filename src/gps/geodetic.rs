#![allow(unused)]
use vqm::{TrigonometricMethods, Vector3df32};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "std", derive(derive_more::Display))]
#[cfg_attr(feature = "std", display("Gps{{long:{longitude_degrees}, lat:{latitude_degrees}, alt:{altitude_meters}}}"))]
pub struct GeographicCoordinate {
    pub longitude_degrees: f32,
    pub latitude_degrees: f32,
    pub altitude_meters: f32,
}

impl GeographicCoordinate {
    pub const fn new(longitude_degrees: f32, latitude_degrees: f32, altitude_meters: f32) -> Self {
        Self { longitude_degrees, latitude_degrees, altitude_meters }
    }
}

impl Default for GeographicCoordinate {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Geodetic {
    home: GeographicCoordinate,
    k_latitude: f32,
    k_longitude: f32,
}

impl Geodetic {
    pub const fn new() -> Self {
        Self { home: GeographicCoordinate::new(0.0, 0.0, 0.0), k_latitude: 0.0, k_longitude: 0.0 }
    }
}

impl Default for Geodetic {
    fn default() -> Self {
        Self::new()
    }
}

impl Geodetic {
    pub const WGS84_EQUATORIAL_RADIUS_METERS: f32 = 6_378_137.0;
}

impl Geodetic {
    pub fn set_home(&mut self, median: GeographicCoordinate) {
        self.home = median;

        let home_lat = self.home.latitude_degrees.to_radians();
        // https://en.wikipedia.org/wiki/Geographical_distance
        self.k_latitude = 111_132.09 - 566.05 * (2.0 * home_lat).cos() + 1.20 * (4.0 * home_lat).cos();
        self.k_longitude =
            111_415.13 * (home_lat).cos() - 94.55 * (3.0 * home_lat).cos() + 0.12 * (5.0 * home_lat).cos();
    }

    pub fn latitude_distance_meters(self, delta_latitude: f32) -> f32 {
        self.k_latitude * delta_latitude
    }

    pub fn longitude_distance_meters(self, delta_longitude: f32) -> f32 {
        self.k_longitude * delta_longitude
    }

    pub fn distance_meters(self, from: GeographicCoordinate, to: GeographicCoordinate) -> Vector3df32 {
        Vector3df32 {
            x: self.k_latitude * (to.latitude_degrees - from.latitude_degrees),
            y: self.k_longitude * (to.longitude_degrees - from.longitude_degrees),
            z: to.altitude_meters - from.altitude_meters,
        }
    }

    pub fn distance_from_home_meters(self, geographic_coordinate: GeographicCoordinate) -> Vector3df32 {
        self.distance_meters(self.home, geographic_coordinate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<GeographicCoordinate>();
        is_full::<Geodetic>();
    }
}
