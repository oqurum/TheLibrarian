pub fn cmp_opt_string(
    old: Option<String>,
    new: Option<String>,
) -> (Option<String>, Option<String>) {
    let old_exists = old.is_some();
    let both_equal = new == old;

    let mut old_out = None;
    let mut new_out = None;

    // If we have an old value AND (the new value doesn't exist OR the new value DOES NOT EQUAL the old value)
    // - Some(old) = OLD & NEW & NEQ

    if old_exists && new.is_some() && !both_equal {
        old_out = old;
    }

    // If we have an new value AND new val is not empty AND (the old value IS NULL OR new DOES NOT EQUAL old)
    // - Some(new) = NEW & !empty & !OLD
    // - Some(new) = NEW & !empty & NEQ

    // TODO: use new.is_some_with
    if new.is_some()
        && new.as_deref().map(|v| !v.is_empty()).unwrap_or_default()
        && (!old_exists || !both_equal)
    {
        new_out = new;
    }

    (old_out, new_out)
}

pub fn cmp_opt_partial_eq<V: std::cmp::PartialEq>(
    old: Option<V>,
    new: Option<V>,
) -> (Option<V>, Option<V>) {
    let old_exists = old.is_some();
    let both_equal = new == old;

    let mut old_out = None;
    let mut new_out = None;

    // If we have an old value AND (the new value doesn't exist OR the new value DOES NOT EQUAL the old value)
    // - Some(old) = OLD & NEW & NEQ

    if old_exists && new.is_some() && !both_equal {
        old_out = old;
    }

    // If we have an new value AND new val is not empty AND (the old value IS NULL OR new DOES NOT EQUAL old)
    // - Some(new) = NEW & !OLD
    // - Some(new) = NEW & NEQ

    if new.is_some() && (!old_exists || !both_equal) {
        new_out = new;
    }

    (old_out, new_out)
}

pub fn cmp_opt_bool(old: Option<bool>, new: Option<bool>) -> (Option<bool>, Option<bool>) {
    let old_exists = old.is_some();
    let both_equal = new == old;

    let mut old_out = None;
    let mut new_out = None;

    // If we have an old value AND (the new value doesn't exist OR the new value DOES NOT EQUAL the old value)
    // - Some(old) = OLD & NEW & NEQ

    if old_exists && new.is_some() && !both_equal {
        old_out = old;
    }

    // If we have an new value AND new val is not empty AND (the old value IS NULL OR new DOES NOT EQUAL old)
    // - Some(new) = NEW & !OLD
    // - Some(new) = NEW & NEQ

    if new.is_some() && (!old_exists || !both_equal) {
        new_out = new;
    }

    (old_out, new_out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_opt_string() {
        // We have both a new and old value. We'll just output it.
        assert_eq!(
            cmp_opt_string(Some(String::from("Old")), Some(String::from("New"))),
            (Some(String::from("Old")), Some(String::from("New")))
        );

        assert_eq!(
            cmp_opt_string(Some(String::from("Old")), Some(String::from("Old"))),
            (None, None)
        );

        assert_eq!(
            cmp_opt_string(Some(String::from("Old")), Some(String::from(""))),
            (Some(String::from("Old")), None)
        );

        assert_eq!(
            cmp_opt_string(Some(String::from("Old")), None),
            (Some(String::from("Old")), None)
        );

        assert_eq!(
            cmp_opt_string(None, Some(String::from("New"))),
            (None, Some(String::from("New")))
        );
    }
}
