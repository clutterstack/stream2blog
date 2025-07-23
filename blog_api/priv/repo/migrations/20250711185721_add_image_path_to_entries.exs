defmodule BlogApi.Repo.Migrations.AddImagePathToEntries do
  use Ecto.Migration

  def change do
    alter table(:entries) do
      add :image_path, :string, null: true
    end
  end
end