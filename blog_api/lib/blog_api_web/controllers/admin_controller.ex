defmodule BlogApiWeb.AdminController do
  use BlogApiWeb, :controller
  require Logger

  @backup_dir "../backups"

  def backup(conn, _params) do
    case create_backup() do
      {:ok, backup_info} ->
        Logger.info("Database backup created: #{backup_info.filename}")
        json(conn, %{data: backup_info})
      
      {:error, reason} ->
        Logger.error("Backup failed: #{reason}")
        conn
        |> put_status(:internal_server_error)
        |> json(%{errors: %{detail: "Backup failed: #{reason}"}})
    end
  end

  def list_backups(conn, _params) do
    case get_backup_list() do
      {:ok, backups} ->
        json(conn, %{data: backups})
      
      {:error, reason} ->
        conn
        |> put_status(:internal_server_error)
        |> json(%{errors: %{detail: "Failed to list backups: #{reason}"}})
    end
  end

  def delete_backup(conn, %{"filename" => filename}) do
    case delete_backup_file(filename) do
      :ok ->
        Logger.info("Backup deleted: #{filename}")
        send_resp(conn, :no_content, "")
      
      {:error, :not_found} ->
        conn
        |> put_status(:not_found)
        |> json(%{errors: %{detail: "Backup not found"}})
      
      {:error, reason} ->
        Logger.error("Failed to delete backup #{filename}: #{reason}")
        conn
        |> put_status(:internal_server_error)
        |> json(%{errors: %{detail: "Failed to delete backup: #{reason}"}})
    end
  end

  defp create_backup do
    # Ensure backup directory exists
    File.mkdir_p!(@backup_dir)
    
    # Cleanup old backups first
    cleanup_old_backups()
    
    # Force WAL checkpoint to ensure all data is in main database file
    case Ecto.Adapters.SQL.query(BlogApi.Repo, "PRAGMA wal_checkpoint(FULL)", []) do
      {:ok, _result} ->
        # Generate backup filename with timestamp
        timestamp = DateTime.utc_now() |> DateTime.to_iso8601(:basic) |> String.replace(":", "-")
        backup_filename = "blog_api_dev_#{timestamp}.db"
        
        # Get database path from repo config
        database_path = get_database_path()
        backup_path = Path.join(@backup_dir, backup_filename)
        
        # Copy database file
        case File.cp(database_path, backup_path) do
          :ok ->
            # Get file info
            case File.stat(backup_path) do
              {:ok, %File.Stat{size: size, mtime: mtime}} ->
                {:ok, %{
                  filename: backup_filename,
                  path: backup_path,
                  size: size,
                  created_at: NaiveDateTime.from_erl!(mtime) |> DateTime.from_naive!("Etc/UTC") |> DateTime.to_iso8601()
                }}
              
              {:error, reason} ->
                {:error, "Failed to get backup file info: #{reason}"}
            end
          
          {:error, reason} ->
            {:error, "Failed to copy database file: #{reason}"}
        end
      
      {:error, reason} ->
        {:error, "Failed to checkpoint WAL: #{reason}"}
    end
  end

  defp get_backup_list do
    case File.ls(@backup_dir) do
      {:ok, files} ->
        backups = 
          files
          |> Enum.filter(&String.ends_with?(&1, ".db"))
          |> Enum.filter(&String.starts_with?(&1, "blog_api_dev_"))
          |> Enum.map(&get_backup_info/1)
          |> Enum.reject(&is_nil/1)
          |> Enum.sort_by(& &1.created_at_datetime, {:desc, DateTime})
        
        {:ok, backups}
      
      {:error, :enoent} ->
        {:ok, []}
      
      {:error, reason} ->
        {:error, reason}
    end
  end

  defp get_backup_info(filename) do
    backup_path = Path.join(@backup_dir, filename)
    
    case File.stat(backup_path) do
      {:ok, %File.Stat{size: size, mtime: mtime}} ->
        datetime = NaiveDateTime.from_erl!(mtime) |> DateTime.from_naive!("Etc/UTC")
        %{
          filename: filename,
          size: size,
          created_at: DateTime.to_iso8601(datetime),
          created_at_datetime: datetime
        }
      
      {:error, _reason} ->
        nil
    end
  end

  defp delete_backup_file(filename) do
    # Validate filename to prevent directory traversal
    if String.contains?(filename, ["../", "..", "/"]) do
      {:error, :invalid_filename}
    else
      backup_path = Path.join(@backup_dir, filename)
      
      case File.exists?(backup_path) do
        true ->
          case File.rm(backup_path) do
            :ok -> :ok
            {:error, reason} -> {:error, reason}
          end
        
        false ->
          {:error, :not_found}
      end
    end
  end

  defp cleanup_old_backups do
    case get_backup_list() do
      {:ok, backups} ->
        # Keep only backups from the last 7 days
        cutoff_date = DateTime.utc_now() |> DateTime.add(-7, :day)
        
        backups
        |> Enum.filter(fn backup ->
          case DateTime.from_iso8601(backup.created_at) do
            {:ok, created_at, _offset} ->
              DateTime.compare(created_at, cutoff_date) == :lt
            
            _ ->
              false
          end
        end)
        |> Enum.each(fn backup ->
          case delete_backup_file(backup.filename) do
            :ok ->
              Logger.info("Removed old backup: #{backup.filename}")
            
            {:error, reason} ->
              Logger.warning("Failed to remove old backup #{backup.filename}: #{reason}")
          end
        end)
      
      {:error, reason} ->
        Logger.warning("Failed to cleanup old backups: #{reason}")
    end
  end

  defp get_database_path do
    # Get database path from repo configuration
    config = BlogApi.Repo.config()
    
    case Keyword.get(config, :database) do
      nil ->
        "blog_api_dev.db"  # fallback
      
      db_path when is_binary(db_path) ->
        if Path.absname(db_path) == db_path do
          db_path  # already absolute
        else
          # Relative path, make it relative to the app
          db_path
        end
    end
  end
end