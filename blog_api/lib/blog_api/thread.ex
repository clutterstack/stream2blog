defmodule BlogApi.Thread do
  use Ecto.Schema
  import Ecto.Changeset

  @primary_key {:id, :binary_id, autogenerate: true}
  @foreign_key_type :binary_id
  schema "threads" do
    field :title, :string
    
    has_many :entries, BlogApi.Entry

    timestamps(type: :utc_datetime)
  end

  @doc false
  def changeset(thread, attrs) do
    thread
    |> cast(attrs, [:title])
    |> validate_required([:title])
  end
end
