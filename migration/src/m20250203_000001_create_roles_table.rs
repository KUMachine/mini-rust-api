use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create the roles table
        manager
            .create_table(
                Table::create()
                    .table(Roles::Table)
                    .if_not_exists()
                    .col(pk_auto(Roles::Id))
                    .col(string_uniq(Roles::Name))
                    .to_owned(),
            )
            .await?;

        // Seed with fixed roles
        let insert = Query::insert()
            .into_table(Roles::Table)
            .columns([Roles::Name])
            .values_panic(["admin".into()])
            .values_panic(["user".into()])
            .to_owned();

        manager.exec_stmt(insert).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Roles::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Roles {
    Table,
    Id,
    Name,
}
