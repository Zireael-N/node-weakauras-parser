// Taken from serde_json
macro_rules! check_recursion {
    ($self:ident, $($body:tt)*) => {
        check_recursion!($self.remaining_depth, $($body)*)
    };
    ($self:ident.$counter:ident, $($body:tt)*) => {
        $self.$counter -= 1;
        if $self.$counter == 0 {
            return Err("Recursion limit exceeded");
        }

        $($body)*

        $self.$counter += 1;
    };
}

pub(crate) use check_recursion;
