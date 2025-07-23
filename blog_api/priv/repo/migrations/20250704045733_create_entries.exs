defmodule BlogApi.Repo.Migrations.CreateEntries do
  use Ecto.Migration

  def change do
    create table(:entries, primary_key: false) do
      add :id, :binary_id, primary_key: true
      add :content, :text
      add :order_num, :integer
      add :image_path, :string, null: true
      add :thread_id, references(:threads, on_delete: :nothing, type: :binary_id)

      timestamps(type: :utc_datetime)
    end

    create index(:entries, [:thread_id])
  end
end
