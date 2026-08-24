Testing submodule for testing that certain migrations behave as expected.
We try to avoid using vault functions, and instead opt for helper query functions without prepared sqlx, in order to avoid adding json schemas, and avoid the tests breaking after future schema changes.
These are relatively disposable and can be removed if necessary after the migration they test has been applied in prod.
