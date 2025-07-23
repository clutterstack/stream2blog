defmodule BlogApiWeb.ThreadController do
  use BlogApiWeb, :controller

  alias BlogApi.{Thread, Entry, Repo}
  import Ecto.Query
  alias Ecto.Multi

  action_fallback BlogApiWeb.FallbackController

  def index(conn, _params) do
    threads = Repo.all(Thread) |> Repo.preload(entries: from(e in Entry, order_by: e.order_num))
    render(conn, :index, threads: threads)
  end

  def create(conn, %{"thread" => thread_params}) do
    changeset = Thread.changeset(%Thread{}, thread_params)
    
    case Repo.insert(changeset) do
      {:ok, thread} ->
        conn
        |> put_status(:created)
        |> put_resp_header("location", ~p"/api/threads/#{thread}")
        |> render(:show, thread: thread)
      {:error, changeset} ->
        conn
        |> put_status(:unprocessable_entity)
        |> render(:error, changeset: changeset)
    end
  end

  def show(conn, %{"id" => id}) do
    thread = Repo.get!(Thread, id) |> Repo.preload(entries: from(e in Entry, order_by: e.order_num))
    render(conn, :show, thread: thread)
  end

  def update(conn, %{"id" => id, "thread" => thread_params}) do
    thread = Repo.get!(Thread, id)
    changeset = Thread.changeset(thread, thread_params)

    case Repo.update(changeset) do
      {:ok, thread} ->
        render(conn, :show, thread: thread)
      {:error, changeset} ->
        conn
        |> put_status(:unprocessable_entity)
        |> render(:error, changeset: changeset)
    end
  end

  def delete(conn, %{"id" => id}) do
    thread = Repo.get!(Thread, id)
    
    case Repo.delete(thread) do
      {:ok, _thread} ->
        send_resp(conn, :no_content, "")
      {:error, changeset} ->
        conn
        |> put_status(:unprocessable_entity)
        |> render(:error, changeset: changeset)
    end
  end

  def export(conn, %{"thread_id" => id}) do
    thread = Repo.get!(Thread, id) |> Repo.preload(entries: from(e in BlogApi.Entry, order_by: e.order_num))
    
    markdown_content = generate_markdown(thread)
    
    conn
    |> put_resp_content_type("text/markdown")
    |> put_resp_header("content-disposition", "attachment; filename=\"#{sanitize_filename(thread.title)}.md\"")
    |> send_resp(200, markdown_content)
  end

  def reorder_entries(conn, %{"thread_id" => thread_id, "entries" => entries}) do
    # Validate that the thread exists
    thread = Repo.get!(Thread, thread_id)
    
    # Update entries within a transaction
    result = Repo.transaction(fn ->
      entries
      |> Enum.each(fn entry_data ->
        entry_id = entry_data["id"]
        new_order = entry_data["order_num"]
        
        entry = Repo.get!(Entry, entry_id)
        changeset = Entry.changeset(entry, %{order_num: new_order})
        
        case Repo.update(changeset) do
          {:ok, _entry} -> :ok
          {:error, changeset} -> Repo.rollback(changeset)
        end
      end)
      
      :ok
    end)
    
    case result do
      {:ok, _} ->
        # Return the updated thread with reordered entries
        updated_thread = Repo.get!(Thread, thread_id) |> Repo.preload(entries: from(e in Entry, order_by: e.order_num))
        render(conn, :show, thread: updated_thread)
      {:error, changeset} ->
        conn
        |> put_status(:unprocessable_entity)
        |> render(:error, changeset: changeset)
    end
  end

  defp generate_markdown(thread) do
    title_line = "# #{thread.title}\n\n"
    
    entries_content = 
      thread.entries
      |> Enum.map(fn entry ->
        # Each entry becomes a paragraph in the markdown
        content = "#{entry.content}\n\n"
        
        # Add image after the text if image_path exists
        if entry.image_path do
          content <> "![Image](#{entry.image_path})\n\n"
        else
          content
        end
      end)
      |> Enum.join("")
    
    title_line <> entries_content
  end

  defp sanitize_filename(title) do
    title
    |> String.replace(~r/[^\w\s-]/, "")
    |> String.replace(~r/\s+/, "_")
    |> String.downcase()
  end
end
