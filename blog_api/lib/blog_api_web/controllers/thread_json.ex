defmodule BlogApiWeb.ThreadJSON do
  alias BlogApi.Thread

  @doc """
  Renders a list of threads.
  """
  def index(%{threads: threads}) do
    %{data: for(thread <- threads, do: data(thread))}
  end

  @doc """
  Renders a single thread.
  """
  def show(%{thread: thread}) do
    %{data: data(thread)}
  end

  defp data(%Thread{} = thread) do
    %{
      id: thread.id,
      title: thread.title,
      inserted_at: thread.inserted_at,
      updated_at: thread.updated_at,
      entries: render_entries(thread.entries)
    }
  end

  defp render_entries(entries) when is_list(entries) do
    Enum.map(entries, fn entry ->
      %{
        id: entry.id,
        content: entry.content,
        order_num: entry.order_num,
        image_path: entry.image_path,
        inserted_at: entry.inserted_at,
        updated_at: entry.updated_at
      }
    end)
  end

  defp render_entries(_), do: []
end
