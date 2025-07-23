defmodule BlogApi.Repo.Migrations.FixEntriesForeignKey do
  use Ecto.Migration

  def change do
    # SQLite doesn't support ALTER TABLE DROP CONSTRAINT, so we recreate the table
    
    # Create new table with correct foreign key constraint
    create table(:entries_new, primary_key: false) do
      add :id, :binary_id, primary_key: true
      add :content, :text
      add :order_num, :integer
      add :thread_id, references(:threads, on_delete: :delete_all, type: :binary_id)
      add :inserted_at, :utc_datetime, null: false
      add :updated_at, :utc_datetime, null: false
    end

    # Copy data from old table to new table
    execute """
    INSERT INTO entries_new (id, content, order_num, thread_id, inserted_at, updated_at)
    SELECT id, content, order_num, thread_id, inserted_at, updated_at
    FROM entries
    """

    # Drop old table and rename new table
    drop table(:entries)
    rename table(:entries_new), to: table(:entries)

    # Recreate index
    create index(:entries, [:thread_id])
  end
end
