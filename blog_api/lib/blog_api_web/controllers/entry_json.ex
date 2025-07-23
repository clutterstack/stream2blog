defmodule BlogApiWeb.EntryJSON do
  alias BlogApi.Entry

  @doc """
  Renders a list of entries.
  """
  def index(%{entries: entries}) do
    %{data: for(entry <- entries, do: data(entry))}
  end

  @doc """
  Renders a single entry.
  """
  def show(%{entry: entry}) do
    %{data: data(entry)}
  end

  defp data(%Entry{} = entry) do
    %{
      id: entry.id,
      content: entry.content,
      order_num: entry.order_num,
      image_path: entry.image_path,
      thread_id: entry.thread_id,
      inserted_at: entry.inserted_at,
      updated_at: entry.updated_at
    }
  end
end
