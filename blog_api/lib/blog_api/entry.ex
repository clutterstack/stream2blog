defmodule BlogApi.Entry do
  use Ecto.Schema
  import Ecto.Changeset

  @primary_key {:id, :binary_id, autogenerate: true}
  @foreign_key_type :binary_id
  schema "entries" do
    field :content, :string
    field :order_num, :integer
    field :image_path, :string
    
    belongs_to :thread, BlogApi.Thread

    timestamps(type: :utc_datetime)
  end

  @doc false
  def changeset(entry, attrs) do
    entry
    |> cast(attrs, [:content, :order_num, :image_path, :thread_id])
    |> validate_required([:content, :order_num, :thread_id])
  end
end
