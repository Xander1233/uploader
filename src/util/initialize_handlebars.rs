use handlebars::Handlebars;

pub async fn init_handlebars<'r>() -> Handlebars<'r> {
    let mut hbs = Handlebars::new();

    hbs.register_template_file("file", "views/file.hbs")
        .unwrap();

    hbs.register_template_file(
        "password_change_notification",
        "views/password_change.email.hbs",
    )
    .unwrap();

    hbs.register_template_file(
        "new_email_changed_notification",
        "views/new_email_changed_notification.email.hbs",
    )
    .unwrap();

    hbs.register_template_file(
        "old_email_changed_notification",
        "views/old_email_changed_notification.email.hbs",
    )
    .unwrap();

    hbs.register_template_file("verify_email", "views/verify_email.email.hbs")
        .unwrap();

    hbs
}
