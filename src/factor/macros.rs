// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#[macro_export]
macro_rules! impl_statement_conversions {
    ($statement_enum:ident, $create_statement:ident, $verify_statement:ident, $final_type:ty) => {
        impl From<super::Mode> for $statement_enum {
            fn from(value: super::Mode) -> Self {
                match value {
                    super::Mode::Create => $statement_enum::Create(Default::default()),
                    super::Mode::Verify => $statement_enum::Verify(Default::default()),
                }
            }
        }

        impl From<$create_statement> for $statement_enum {
            fn from(value: $create_statement) -> Self {
                Self::Create(value)
            }
        }

        impl From<$verify_statement> for $statement_enum {
            fn from(value: $verify_statement) -> Self {
                Self::Verify(value)
            }
        }

        impl TryInto<$create_statement> for $statement_enum {
            type Error = ();

            fn try_into(self) -> Result<$create_statement, Self::Error> {
                match self {
                    $statement_enum::Create(state) => Ok(state),
                    _ => Err(()),
                }
            }
        }

        impl TryInto<$verify_statement> for $statement_enum {
            type Error = ();

            fn try_into(self) -> Result<$verify_statement, Self::Error> {
                match self {
                    $statement_enum::Verify(state) => Ok(state),
                    _ => Err(()),
                }
            }
        }

        impl crate::factor::macros::MaybeMut<$statement_enum> for $create_statement {
            fn maybe_mut<'a>(other: &'a mut $statement_enum) -> Option<&'a mut Self> {
                match other {
                    $statement_enum::Create(state) => Some(state),
                    _ => None,
                }
            }
        }

        impl crate::factor::macros::MaybeMut<$statement_enum> for $verify_statement {
            fn maybe_mut<'a>(other: &'a mut $statement_enum) -> Option<&'a mut Self> {
                match other {
                    $statement_enum::Verify(state) => Some(state),
                    _ => None,
                }
            }
        }

        impl crate::factor::macros::TakeFinalValue for $statement_enum {
            type Final = $final_type;

            fn take_final(&mut self) -> Self::Final {
                match self {
                    $statement_enum::Create(state) => match state {
                        $create_statement::Final(v) => v.take(),
                        _ => None,
                    },
                    $statement_enum::Verify(state) => match state {
                        $verify_statement::Final(v) => v.take(),
                        _ => None,
                    },
                }
            }
        }
    };
}

pub trait MaybeMut<T> {
    fn maybe_mut<'a>(other: &'a mut T) -> Option<&'a mut Self>;
}

pub trait TakeFinalValue {
    type Final;
    fn take_final(&mut self) -> Self::Final;
}
