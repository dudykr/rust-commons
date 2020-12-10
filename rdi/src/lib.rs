pub use rdi_macros::{inject, injector};

/// **Do not implement this manually**
pub trait Injectable<'a> {
    type Output: 'a;
    type Injected: 'a;

    fn inject(self, injected: Self::Injected) -> Self::Output;
}

/// **Do not implement this manually**
pub trait Provider<T> {
    fn provide(&self) -> T;
}

macro_rules! impl_provider {
    (
        $(
            $name:ident
        ),*
    ) => {
        impl<$($name),*, P> Provider<($($name,)*)> for P
        where
            $(
                Self: Value<$name>,
            )*
        {
            fn provide(&self) -> ($($name,)*) {
                let provided: ($($name,)*) = ($(Value::<$name>::value(self),)*);
                provided
            }
        }
    };
}

impl_provider!(A);
impl_provider!(A, B);
impl_provider!(A, B, C);
impl_provider!(A, B, C, D);
impl_provider!(A, B, C, D, E);
impl_provider!(A, B, C, D, E, F);
impl_provider!(A, B, C, D, E, F, G);
impl_provider!(A, B, C, D, E, F, G, H);
impl_provider!(A, B, C, D, E, F, G, H, J);
impl_provider!(A, B, C, D, E, F, G, H, J, K);
impl_provider!(A, B, C, D, E, F, G, H, J, K, L);
impl_provider!(A, B, C, D, E, F, G, H, J, K, L, M);

/// **Do not implement this manually**. Use `#[injector]` instead.
pub trait Value<T> {
    fn value(&self) -> T;
}

#[cfg(test)]
mod tests {
    use super::*;
    // We test if it works without macro.

    struct OkInjector {
        test: Test,
    }

    struct Test {}

    impl Value<Test> for OkInjector {
        fn value(&self) -> Test {
            self.test
        }
    }

    impl OkInjector {
        fn inject<'a, T>(&self, t: T) -> T::Output
        where
            Self: Provider<T::Injected>,
            T: Injectable<'a>,
        {
            let injected = self.provide();
            t.inject(self, injected)
        }
    }

    #[allow(non_camel_case_types)]
    struct handler;

    impl<'a> Injectable<'a> for handler {
        type Output = &'a dyn Fn();
        type Injected = (Test,);

        fn inject(self, injected: Self::Injected) -> Self::Output {
            &|| {}
        }
    }

    fn test() {
        let mut injector = OkInjector { test: Test {} };
        let my_handler = injector.inject(handler);
        my_handler();
    }
}
