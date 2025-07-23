defmodule BlogApiWeb.EntryControllerTest do
  use BlogApiWeb.ConnCase

  import BlogApi.BlogFixtures
  alias BlogApi.Blog.Entry

  @create_attrs %{
    content: "some content",
    order_num: 42
  }
  @update_attrs %{
    content: "some updated content",
    order_num: 43
  }
  @invalid_attrs %{content: nil, order_num: nil}

  setup %{conn: conn} do
    {:ok, conn: put_req_header(conn, "accept", "application/json")}
  end

  describe "index" do
    test "lists all entries", %{conn: conn} do
      conn = get(conn, ~p"/api/entries")
      assert json_response(conn, 200)["data"] == []
    end
  end

  describe "create entry" do
    test "renders entry when data is valid", %{conn: conn} do
      conn = post(conn, ~p"/api/entries", entry: @create_attrs)
      assert %{"id" => id} = json_response(conn, 201)["data"]

      conn = get(conn, ~p"/api/entries/#{id}")

      assert %{
               "id" => ^id,
               "content" => "some content",
               "order_num" => 42
             } = json_response(conn, 200)["data"]
    end

    test "renders errors when data is invalid", %{conn: conn} do
      conn = post(conn, ~p"/api/entries", entry: @invalid_attrs)
      assert json_response(conn, 422)["errors"] != %{}
    end
  end

  describe "update entry" do
    setup [:create_entry]

    test "renders entry when data is valid", %{conn: conn, entry: %Entry{id: id} = entry} do
      conn = put(conn, ~p"/api/entries/#{entry}", entry: @update_attrs)
      assert %{"id" => ^id} = json_response(conn, 200)["data"]

      conn = get(conn, ~p"/api/entries/#{id}")

      assert %{
               "id" => ^id,
               "content" => "some updated content",
               "order_num" => 43
             } = json_response(conn, 200)["data"]
    end

    test "renders errors when data is invalid", %{conn: conn, entry: entry} do
      conn = put(conn, ~p"/api/entries/#{entry}", entry: @invalid_attrs)
      assert json_response(conn, 422)["errors"] != %{}
    end
  end

  describe "delete entry" do
    setup [:create_entry]

    test "deletes chosen entry", %{conn: conn, entry: entry} do
      conn = delete(conn, ~p"/api/entries/#{entry}")
      assert response(conn, 204)

      assert_error_sent 404, fn ->
        get(conn, ~p"/api/entries/#{entry}")
      end
    end
  end

  defp create_entry(_) do
    entry = entry_fixture()

    %{entry: entry}
  end
end
