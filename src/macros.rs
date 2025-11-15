#[macro_export]
macro_rules! type_expr {
    ($ty:ty, $expr:expr) => {{
        let e: $ty = $expr;
        e
    }};
}

#[macro_export]
macro_rules! or_return {
    ($ret_val:ident => $ret:block : Option($($tail:tt)*)) => {
        match (or_return!($ret_val => $ret : $($tail)*)) {
            Some(res) => res,
            None => {
                let $ret_val = Option::<()>::None;
                $ret
            }
        }
    };
    ($ret_val:ident => $ret:block : Result($($tail:tt)*)) => {
        match (or_return!($ret_val => $ret : $($tail)*)) {
            Ok(res) => res,
            Err(__err) => {
                let $ret_val = __err;
                $ret
            }
        }
    };
    ($ret_val:ident => $ret:block : $expr:expr) => {
        $expr
    };
    ($($tail:tt)*) => {
        or_return!(_r => {return;} : $($tail)*)
    };
}

#[macro_export]
macro_rules! capture_result {
    {$($block:tt)*} => {
        {
            let res = || {
                $($block)*
            };
            res()
        }
    }
}
