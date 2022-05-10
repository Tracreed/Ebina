#[macro_export]
macro_rules! anilist_embed {
    ($cond:expr, $title:expr, $emb:expr) => {
        if $cond.is_some() {
            $emb.field($title, $cond.unwrap(), true);
        }
    };
    ($cond:expr, $value:expr, $title:expr, $emb:expr) => {
        if $cond {
            $emb.field($title, $value, true);
        }
    };
}

/// Takes in a context and adds 1 to command usage
#[macro_export]
macro_rules! add_command_tracking {
	($ctx:expr, $name:expr) => {
		let mut __data = $ctx.data.write().await;
		let __counter = __data.get_mut::<ebina_types::CommandCounter>().expect("Expected CommandCounter in TypeMap");
		let __entry = __counter.entry($name.into()).or_insert(0);
		*__entry += 1;
	};
}
