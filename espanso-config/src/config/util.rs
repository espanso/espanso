#[macro_export]
macro_rules! merge {
  ( $t:ident, $child:expr, $parent:expr, $( $x:ident ),* ) => {
    {
      $(
        if $child.$x.is_none() {
          $child.$x = $parent.$x.clone();
        }
      )*
      
      // Build a temporary object to verify that all fields
      // are being used at compile time
      $t {
        $(
          $x: None,
        )*
      };
    }
  };
}
