defmodule BlogApi.Repo do
  use Ecto.Repo,
    otp_app: :blog_api,
    adapter: Ecto.Adapters.SQLite3
end
