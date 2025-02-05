use std::borrow::Cow;

pub trait Convert<V: Clone> {
    /// converts a value based on the type of this object and
    /// the "to" object.
    /// in the domain of numeric units, this is implemented by
    /// unit type enums, such as DistanceUnit for Distance values:
    ///
    /// ```rust
    /// use super::{Distance, DistanceUnit};
    /// let mut distance = Distance::new(1000.0);
    /// let kilometers = DistanceUnit::Kilometers;
    /// DistanceUnit::Meters.convert(&distance, &kilometers);
    /// assert_eq!(distance, Distance::ONE);
    ///
    /// # Arguments
    /// * `value` - the value to possibly convert, wrapped in a
    ///             copy-on-write (Cow) smart pointer
    /// * `to`    - the unit type to convert to.
    /// ```
    fn convert(&self, value: &mut Cow<V>, to: &Self);

    /// converts a value based on the type of this object into
    /// the base unit type used within the Compass models.
    ///
    /// base units are defined in [`super::base_unit_ops`]
    ///
    /// in the domain of numeric units, this is implemented by
    /// unit type enums, such as DistanceUnit for Distance values:
    ///
    /// ```rust
    /// use super::{Distance, DistanceUnit};
    /// let mut distance = Distance::new(1.0);
    /// DistanceUnit::Kilometers.convert_to_base(&distance);
    /// assert_eq!(distance, Distance::new(1000.0));
    ///
    /// # Arguments
    /// * `value` - the value to possibly convert, wrapped in a
    ///             copy-on-write (Cow) smart pointer
    /// ```
    fn convert_to_base(&self, value: &mut Cow<V>);
}
