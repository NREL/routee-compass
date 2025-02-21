use std::borrow::Cow;

use super::UnitError;

pub trait Convert<V: Clone> {
    /// converts a value based on the type of this object and
    /// the "to" object.
    /// in the domain of numeric units, this is implemented by
    /// unit type enums, such as DistanceUnit for Distance values:
    ///
    /// ```rust
    /// use std::borrow::Cow;
    /// use routee_compass_core::model::unit::{Distance, DistanceUnit, Convert};
    /// let mut distance: Cow<Distance> = Cow::Owned(Distance::from(1000.0));
    /// let kilometers = DistanceUnit::Kilometers;
    /// DistanceUnit::Meters.convert(&mut distance, &kilometers);
    /// assert_eq!(distance.into_owned(), Distance::ONE);
    /// ```
    /// # Arguments
    /// * `value` - the value to possibly convert, wrapped in a
    ///             copy-on-write (Cow) smart pointer
    /// * `to`    - the unit type to convert to.
    fn convert(&self, value: &mut Cow<V>, to: &Self) -> Result<(), UnitError>;

    /// converts a value based on the type of this object into
    /// the base unit type used within the Compass models.
    ///
    /// base units are defined in [`super::base_unit_ops`]
    ///
    /// in the domain of numeric units, this is implemented by
    /// unit type enums, such as DistanceUnit for Distance values:
    ///
    /// ```rust
    /// use std::borrow::Cow;
    /// use routee_compass_core::model::unit::{Distance, DistanceUnit, Convert};
    /// let mut distance: Cow<Distance> = Cow::Owned(Distance::from(1.0));
    /// DistanceUnit::Kilometers.convert_to_base(&mut distance);
    /// assert_eq!(distance.into_owned(), Distance::from(1000.0));
    /// ```
    ///
    /// # Arguments
    /// * `value` - the value to possibly convert, wrapped in a
    ///             copy-on-write (Cow) smart pointer
    fn convert_to_base(&self, value: &mut Cow<V>) -> Result<(), UnitError>;
}
