defmodule BlogApiWeb.EntryController do
  use BlogApiWeb, :controller

  alias BlogApi.{Entry, Repo}

  action_fallback BlogApiWeb.FallbackController

  def index(conn, _params) do
    entries = Repo.all(Entry)
    render(conn, :index, entries: entries)
  end

  def create(conn, %{"entry" => entry_params}) do
    changeset = Entry.changeset(%Entry{}, entry_params)
    
    case Repo.insert(changeset) do
      {:ok, entry} ->
        conn
        |> put_status(:created)
        |> put_resp_header("location", ~p"/api/entries/#{entry}")
        |> render(:show, entry: entry)
      {:error, changeset} ->
        conn
        |> put_status(:unprocessable_entity)
        |> render(:error, changeset: changeset)
    end
  end

  def show(conn, %{"id" => id}) do
    entry = Repo.get!(Entry, id)
    render(conn, :show, entry: entry)
  end

  def update(conn, %{"id" => id, "entry" => entry_params}) do
    entry = Repo.get!(Entry, id)
    changeset = Entry.changeset(entry, entry_params)

    case Repo.update(changeset) do
      {:ok, entry} ->
        render(conn, :show, entry: entry)
      {:error, changeset} ->
        conn
        |> put_status(:unprocessable_entity)
        |> render(:error, changeset: changeset)
    end
  end

  def delete(conn, %{"id" => id}) do
    entry = Repo.get!(Entry, id)
    
    case Repo.delete(entry) do
      {:ok, _entry} ->
        send_resp(conn, :no_content, "")
      {:error, changeset} ->
        conn
        |> put_status(:unprocessable_entity)
        |> render(:error, changeset: changeset)
    end
  end
end
