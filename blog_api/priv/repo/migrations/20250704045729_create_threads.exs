defmodule BlogApi.Repo.Migrations.CreateThreads do
  use Ecto.Migration

  def change do
    create table(:threads, primary_key: false) do
      add :id, :binary_id, primary_key: true
      add :title, :string

      timestamps(type: :utc_datetime)
    end
  end
end
