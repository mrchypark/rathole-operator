defmodule Rathole.Application do
  # See https://hexdocs.pm/elixir/Application.html
  # for more information on OTP Applications
  @moduledoc false

  use Application

  def start(_type, env: env) do
    opts = [strategy: :one_for_one, name: Rathole.Supervisor]
    Supervisor.start_link(children(env), opts)
  end

  # If you want to implement integration tests, remove the following line:
  defp children(:test), do: []
  defp children(env) do
    [
      {
        Rathole.Operator,
        conn: Rathole.K8sConn.get!(env),
        watch_namespace: :all
      }
    ]
  end
end
