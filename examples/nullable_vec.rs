macro_rules! nullable_vec {
  // (@vec( $( $e:expr, )* ) nil , $($rest:tt)* ) => {
  //     nullable_vec!( @vec($($e,)* None,) $($rest)* )
  // };

  // (@vec( $( $e:expr, )* ) nil ) => {
  //     nullable_vec!( @vec($($e,)* None,) )
  // };

  // (@vec( $( $e:expr, )* ) null, $($rest:tt)* ) => {
  //     nullable_vec!( @vec($($e,)* None,) $($rest)* )
  // };

  // (@vec( $( $e:expr, )* ) null ) => {
  //     nullable_vec!( @vec($($e,)* None,) )
  // };

  (@vec( $( $e:expr, )* ) $v:expr , $($rest:tt)* ) => {
      nullable_vec!( @vec($($e,)* Some($v),) $($rest)* )
  };

  (@vec( $( $e:expr, )* ) $v:expr ) => {
      nullable_vec!( @vec($($e,)* Some($v),) )
  };

  (@vec( $( $e:expr, )* )) => {
      vec![ $($e),* ]
  };

  [ $($inner:tt)* ] => {
      nullable_vec!( @vec() $($inner)* )
  };
}

fn main() {
    let foo = 30;
    let node_values = nullable_vec![3 + 4, 9, foo, foo, (16 - 8), 7, 20 / 5];
    println!("{:?}", node_values);
}
