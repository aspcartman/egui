use crate::{Pos2, Rect};

/// A Core Animation-style homogeneous transform.
///
/// Points are row vectors: `(x, y, z, 1) * transform`. Translation therefore lives in
/// `m41`, `m42`, and `m43`; `m34` is the usual perspective entry.
///
/// # Example
///
/// ```
/// use emath::{pos2, Transform3D};
///
/// let transform = Transform3D::from_translation(10.0, 5.0, 0.0);
/// assert_eq!(transform.transform_pos2(pos2(2.0, 3.0)), Some(pos2(12.0, 8.0)));
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform3D {
    m11: f32,
    m12: f32,
    m13: f32,
    m14: f32,
    m21: f32,
    m22: f32,
    m23: f32,
    m24: f32,
    m31: f32,
    m32: f32,
    m33: f32,
    m34: f32,
    m41: f32,
    m42: f32,
    m43: f32,
    m44: f32,
}

impl Default for Transform3D {
    #[inline]
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform3D {
    /// The identity transform.
    pub const IDENTITY: Self = Self {
        m11: 1.0,
        m12: 0.0,
        m13: 0.0,
        m14: 0.0,
        m21: 0.0,
        m22: 1.0,
        m23: 0.0,
        m24: 0.0,
        m31: 0.0,
        m32: 0.0,
        m33: 1.0,
        m34: 0.0,
        m41: 0.0,
        m42: 0.0,
        m43: 0.0,
        m44: 1.0,
    };

    /// Returns an affine translation by `(tx, ty, tz)`.
    #[inline]
    pub fn from_translation(tx: f32, ty: f32, tz: f32) -> Self {
        Self {
            m41: tx,
            m42: ty,
            m43: tz,
            ..Self::IDENTITY
        }
    }

    /// Returns an affine scale by `(sx, sy, sz)`.
    #[inline]
    pub fn from_scale(sx: f32, sy: f32, sz: f32) -> Self {
        Self {
            m11: sx,
            m22: sy,
            m33: sz,
            ..Self::IDENTITY
        }
    }

    /// Returns a projective transform with Core Animation's `m34` perspective term.
    #[inline]
    pub fn from_perspective(m34: f32) -> Self {
        Self {
            m34,
            ..Self::IDENTITY
        }
    }

    /// Returns a clockwise axis-angle rotation when looking from the positive axis toward the origin.
    #[inline]
    pub fn from_rotation(angle: f32, x: f32, y: f32, z: f32) -> Self {
        let length = (x * x + y * y + z * z).sqrt();
        let x = x / length;
        let y = y / length;
        let z = z / length;
        let sin = angle.sin();
        let cos = angle.cos();
        let one_minus_cos = 1.0 - cos;

        // This is the transpose of the conventional column-vector rotation matrix.
        Self {
            m11: cos + x * x * one_minus_cos,
            m12: y * x * one_minus_cos + z * sin,
            m13: z * x * one_minus_cos - y * sin,
            m14: 0.0,
            m21: x * y * one_minus_cos - z * sin,
            m22: cos + y * y * one_minus_cos,
            m23: z * y * one_minus_cos + x * sin,
            m24: 0.0,
            m31: x * z * one_minus_cos + y * sin,
            m32: y * z * one_minus_cos - x * sin,
            m33: cos + z * z * one_minus_cos,
            m34: 0.0,
            m41: 0.0,
            m42: 0.0,
            m43: 0.0,
            m44: 1.0,
        }
    }

    /// Returns a rotation around the x axis.
    #[inline]
    pub fn from_rotation_x(angle: f32) -> Self {
        Self::from_rotation(angle, 1.0, 0.0, 0.0)
    }

    /// Returns a rotation around the y axis.
    #[inline]
    pub fn from_rotation_y(angle: f32) -> Self {
        Self::from_rotation(angle, 0.0, 1.0, 0.0)
    }

    /// Returns a clockwise rotation around the z axis.
    #[inline]
    pub fn from_rotation_z(angle: f32) -> Self {
        Self::from_rotation(angle, 0.0, 0.0, 1.0)
    }

    /// Concatenates `rhs` after `self`.
    #[inline]
    pub fn concat(self, rhs: Self) -> Self {
        self * rhs
    }

    /// Concatenates a translation after this transform.
    #[inline]
    pub fn translated(self, tx: f32, ty: f32, tz: f32) -> Self {
        self * Self::from_translation(tx, ty, tz)
    }

    /// Concatenates a scale after this transform.
    #[inline]
    pub fn scaled(self, sx: f32, sy: f32, sz: f32) -> Self {
        self * Self::from_scale(sx, sy, sz)
    }

    /// Concatenates an axis-angle rotation after this transform.
    #[inline]
    pub fn rotated(self, angle: f32, x: f32, y: f32, z: f32) -> Self {
        self * Self::from_rotation(angle, x, y, z)
    }

    /// Concatenates Core Animation's `m34` perspective after this transform.
    #[inline]
    pub fn with_perspective(self, m34: f32) -> Self {
        self * Self::from_perspective(m34)
    }

    /// Returns the x and y scale entries of a planar transform.
    ///
    /// This is primarily useful for a pan-and-zoom transform. It does not decompose a rotation.
    #[inline]
    pub fn scale_2d(&self) -> (f32, f32) {
        (self.m11, self.m22)
    }

    /// Replaces the x and y scale entries of a planar transform.
    ///
    /// This does not decompose or preserve rotation; use it for a pan-and-zoom transform.
    #[inline]
    pub fn with_scale_2d(mut self, sx: f32, sy: f32) -> Self {
        self.m11 = sx;
        self.m22 = sy;
        self
    }

    /// Returns this transform around the given point in the UI plane.
    #[inline]
    pub fn around(self, pivot: Pos2) -> Self {
        Self::from_translation(-pivot.x, -pivot.y, 0.0)
            * self
            * Self::from_translation(pivot.x, pivot.y, 0.0)
    }

    /// Returns `true` if this is exactly the identity transform.
    #[inline]
    pub fn is_identity(&self) -> bool {
        *self == Self::IDENTITY
    }

    /// Returns `true` when this matrix is a two-dimensional affine transform.
    #[inline]
    pub fn is_planar_2d(&self) -> bool {
        self.m13 == 0.0
            && self.m14 == 0.0
            && self.m23 == 0.0
            && self.m24 == 0.0
            && self.m31 == 0.0
            && self.m32 == 0.0
            && self.m33 == 1.0
            && self.m34 == 0.0
            && self.m43 == 0.0
            && self.m44 == 1.0
    }

    /// Returns `true` when this matrix has no projective terms.
    #[inline]
    pub fn is_affine(&self) -> bool {
        self.m14 == 0.0 && self.m24 == 0.0 && self.m34 == 0.0 && self.m44 == 1.0
    }

    /// Transforms a local UI point on the `z = 0` plane, including homogeneous division.
    ///
    /// Returns `None` when the point is non-finite or lies on or behind the projection plane.
    #[inline]
    pub fn transform_pos2(&self, pos: Pos2) -> Option<Pos2> {
        let x = pos.x * self.m11 + pos.y * self.m21 + self.m41;
        let y = pos.x * self.m12 + pos.y * self.m22 + self.m42;
        let w = pos.x * self.m14 + pos.y * self.m24 + self.m44;
        if !x.is_finite() || !y.is_finite() || !w.is_finite() || w <= f32::EPSILON {
            None
        } else {
            Some(Pos2::new(x / w, y / w))
        }
    }

    /// Maps a screen point back onto this transform's local `z = 0` UI plane.
    ///
    /// Returns `None` when the point has no finite preimage in front of the projection plane.
    pub fn unproject_pos2(&self, pos: Pos2) -> Option<Pos2> {
        // Solve the 2D homography induced by the local UI plane directly.
        let a = self.m11 - pos.x * self.m14;
        let b = self.m21 - pos.x * self.m24;
        let c = self.m12 - pos.y * self.m14;
        let d = self.m22 - pos.y * self.m24;
        let x = pos.x * self.m44 - self.m41;
        let y = pos.y * self.m44 - self.m42;
        let determinant = a * d - b * c;
        if !determinant.is_finite() || determinant.abs() <= f32::EPSILON {
            return None;
        }
        let local = Pos2::new((x * d - b * y) / determinant, (a * y - x * c) / determinant);
        self.transform_pos2(local)
            .filter(|projected| projected.distance_sq(pos) <= 0.01)
            .map(|_| local)
    }

    /// Projects a UI rectangle.
    ///
    /// Returns `None` if the rectangle crosses the projection plane.
    pub fn transform_rect(&self, rect: Rect) -> Option<Rect> {
        let corners = [
            rect.left_top(),
            rect.right_top(),
            rect.right_bottom(),
            rect.left_bottom(),
        ];
        let mut projected = [Pos2::ZERO; 4];
        for (index, point) in corners.into_iter().enumerate() {
            let w = point.x * self.m14 + point.y * self.m24 + self.m44;
            if !w.is_finite() || w <= f32::EPSILON {
                return None;
            }
            projected[index] = self.transform_pos2(point)?;
        }

        Some(Rect::from_points(&projected))
    }

    /// Inverts this matrix using Gauss-Jordan elimination.
    ///
    /// Returns `None` for a non-finite or non-invertible matrix.
    pub fn inverse(&self) -> Option<Self> {
        let mut augmented = [[0.0_f32; 8]; 4];
        for (row, values) in self.as_rows().into_iter().enumerate() {
            augmented[row][..4].copy_from_slice(&values);
            augmented[row][row + 4] = 1.0;
        }

        for pivot in 0..4 {
            let mut pivot_row = pivot;
            for row in pivot + 1..4 {
                if augmented[row][pivot].abs() > augmented[pivot_row][pivot].abs() {
                    pivot_row = row;
                }
            }
            if !augmented[pivot_row][pivot].is_finite()
                || augmented[pivot_row][pivot].abs() <= f32::EPSILON
            {
                return None;
            }
            augmented.swap(pivot, pivot_row);

            let divisor = augmented[pivot][pivot];
            for value in &mut augmented[pivot] {
                *value /= divisor;
            }
            let pivot_values = augmented[pivot];
            for (row, values) in augmented.iter_mut().enumerate() {
                if row == pivot {
                    continue;
                }
                let factor = values[pivot];
                for (value, pivot_value) in values.iter_mut().zip(pivot_values) {
                    *value -= factor * pivot_value;
                }
            }
        }

        let inverse = Self::from_rows_unchecked([
            [
                augmented[0][4],
                augmented[0][5],
                augmented[0][6],
                augmented[0][7],
            ],
            [
                augmented[1][4],
                augmented[1][5],
                augmented[1][6],
                augmented[1][7],
            ],
            [
                augmented[2][4],
                augmented[2][5],
                augmented[2][6],
                augmented[2][7],
            ],
            [
                augmented[3][4],
                augmented[3][5],
                augmented[3][6],
                augmented[3][7],
            ],
        ]);
        inverse.is_finite().then_some(inverse)
    }

    /// Returns the matrix in its Core Animation row-vector order.
    #[inline]
    pub fn as_rows(&self) -> [[f32; 4]; 4] {
        [
            [self.m11, self.m12, self.m13, self.m14],
            [self.m21, self.m22, self.m23, self.m24],
            [self.m31, self.m32, self.m33, self.m34],
            [self.m41, self.m42, self.m43, self.m44],
        ]
    }

    /// Constructs a transform from Core Animation row-vector matrix rows without validation.
    #[inline]
    pub fn from_rows(rows: [[f32; 4]; 4]) -> Self {
        Self::from_rows_unchecked(rows)
    }

    /// Constructs a finite, invertible transform from Core Animation row-vector matrix rows.
    ///
    /// Returns `None` for a non-finite or non-invertible matrix.
    #[inline]
    pub fn try_from_rows(rows: [[f32; 4]; 4]) -> Option<Self> {
        let transform = Self::from_rows(rows);
        transform.is_valid().then_some(transform)
    }

    /// Returns `true` if all matrix entries are finite.
    #[inline]
    pub fn is_finite(&self) -> bool {
        self.as_rows()
            .iter()
            .flatten()
            .all(|value| value.is_finite())
    }

    /// Returns `true` if all matrix entries are finite and the matrix is invertible.
    ///
    /// Use [`Self::transform_rect`] to validate a particular UI rectangle against the projection plane.
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.is_finite() && self.inverse().is_some()
    }

    #[inline]
    fn from_rows_unchecked(rows: [[f32; 4]; 4]) -> Self {
        Self {
            m11: rows[0][0],
            m12: rows[0][1],
            m13: rows[0][2],
            m14: rows[0][3],
            m21: rows[1][0],
            m22: rows[1][1],
            m23: rows[1][2],
            m24: rows[1][3],
            m31: rows[2][0],
            m32: rows[2][1],
            m33: rows[2][2],
            m34: rows[2][3],
            m41: rows[3][0],
            m42: rows[3][1],
            m43: rows[3][2],
            m44: rows[3][3],
        }
    }
}

impl core::ops::Mul for Transform3D {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        let left = self.as_rows();
        let right = rhs.as_rows();
        let mut result = [[0.0_f32; 4]; 4];
        for row in 0..4 {
            for column in 0..4 {
                for index in 0..4 {
                    result[row][column] += left[row][index] * right[index][column];
                }
            }
        }
        Self::from_rows(result)
    }
}

impl core::ops::Mul<Pos2> for Transform3D {
    type Output = Pos2;

    #[inline]
    fn mul(self, pos: Pos2) -> Self::Output {
        self.transform_pos2(pos).unwrap_or(Pos2::NAN)
    }
}

impl core::ops::Mul<Rect> for Transform3D {
    type Output = Rect;

    #[inline]
    fn mul(self, rect: Rect) -> Self::Output {
        self.transform_rect(rect).unwrap_or(Rect::NOTHING)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Transform3D {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde::Serialize::serialize(&self.as_rows(), serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Transform3D {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let rows = <[[f32; 4]; 4] as serde::Deserialize>::deserialize(deserializer)?;
        Ok(Self::from_rows(rows))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translation_uses_core_animation_fields() {
        let transform = Transform3D::from_translation(2.0, 3.0, 4.0);
        assert_eq!(transform.as_rows()[3], [2.0, 3.0, 4.0, 1.0]);
        assert_eq!(
            transform.transform_pos2(Pos2::ZERO),
            Some(Pos2::new(2.0, 3.0))
        );
    }

    #[test]
    fn perspective_projects_the_ui_plane() {
        let perspective = Transform3D::from_perspective(-1.0 / 100.0);
        let transform = Transform3D::from_translation(0.0, 0.0, 10.0) * perspective;
        assert_eq!(
            transform.transform_pos2(Pos2::new(90.0, 0.0)),
            Some(Pos2::new(100.0, 0.0))
        );
        assert_eq!(
            transform.unproject_pos2(Pos2::new(100.0, 0.0)),
            Some(Pos2::new(90.0, 0.0))
        );
    }

    #[test]
    fn inverse_undoes_translation() {
        let transform = Transform3D::from_translation(2.0, 3.0, 0.0);
        let inverse = transform.inverse().unwrap();
        assert_eq!(
            inverse.transform_pos2(Pos2::new(2.0, 3.0)),
            Some(Pos2::ZERO)
        );
    }

    #[test]
    fn concatenation_applies_the_left_transform_first() {
        let transform = Transform3D::from_scale(2.0, 2.0, 1.0)
            .concat(Transform3D::from_translation(3.0, 4.0, 0.0));
        assert_eq!(transform * Pos2::new(1.0, 1.0), Pos2::new(5.0, 6.0));
    }

    #[test]
    fn around_keeps_its_pivot_in_place() {
        let pivot = Pos2::new(4.0, 5.0);
        let transform = Transform3D::from_rotation_z(core::f32::consts::FRAC_PI_2).around(pivot);
        assert!(
            transform
                .transform_pos2(pivot)
                .is_some_and(|projected| projected.distance_sq(pivot) < 1e-5)
        );
    }

    #[test]
    fn rotation_uses_eguis_clockwise_axis_convention() {
        let transform = Transform3D::from_rotation_z(core::f32::consts::FRAC_PI_2);
        assert!(
            transform
                .transform_pos2(Pos2::new(1.0, 0.0))
                .is_some_and(|projected| projected.distance_sq(Pos2::new(0.0, 1.0)) < 1e-5)
        );
    }

    #[test]
    fn classifies_planar_and_projective_transforms() {
        let translation = Transform3D::from_translation(2.0, 3.0, 0.0);
        assert!(translation.is_planar_2d());
        assert!(translation.is_affine());

        let rotation = Transform3D::from_rotation_y(0.2);
        assert!(!rotation.is_planar_2d());
        assert!(rotation.is_affine());

        let perspective = Transform3D::from_perspective(-0.01);
        assert!(!perspective.is_planar_2d());
        assert!(!perspective.is_affine());
    }

    #[test]
    fn rejects_points_behind_or_on_the_projection_plane() {
        let transform = Transform3D::try_from_rows([
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
        .unwrap();
        assert_eq!(transform.transform_pos2(Pos2::new(-1.0, 0.0)), None);
        assert_eq!(
            transform.transform_rect(Rect::from_min_max(
                Pos2::new(-2.0, 0.0),
                Pos2::new(1.0, 1.0)
            )),
            None
        );
    }

    #[test]
    fn validity_reports_malformed_transforms() {
        let singular = Transform3D::from_scale(1.0, 1.0, 0.0);
        assert_ne!(singular, Transform3D::IDENTITY);
        assert_eq!(singular.inverse(), None);
        assert!(!singular.is_valid());

        let non_finite = Transform3D::from_translation(f32::NAN, 0.0, 0.0);
        assert!(!non_finite.is_finite());
        assert!(!non_finite.is_valid());

        assert!(
            Transform3D::try_from_rows([
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ])
            .is_none()
        );
        assert!(
            Transform3D::try_from_rows([
                [f32::NAN, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ])
            .is_none()
        );
    }
}
