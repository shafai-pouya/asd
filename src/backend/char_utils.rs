pub trait MyCharUtils {
    fn is_variable_name(&self) -> bool;
}

impl MyCharUtils for char {
    fn is_variable_name(&self) -> bool {
        self.is_alphanumeric() || self == &'_'
    }
}