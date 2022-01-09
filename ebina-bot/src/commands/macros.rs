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
