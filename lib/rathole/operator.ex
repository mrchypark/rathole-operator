defmodule Rathole.Operator do
  @moduledoc """
  Defines the operator.

  The operator resource defines custom resources, watch queries and their
  controllers and serves as the entry point to the watching and handling
  processes.
  """

  use Bonny.Operator, default_watch_namespace: "default"

  step(Bonny.Pluggable.Logger, level: :info)
  step(:delegate_to_controller)
  step(Bonny.Pluggable.ApplyStatus)
  step(Bonny.Pluggable.ApplyDescendants)

  @impl Bonny.Operator
  def controllers(watching_namespace, _opts) do
    [
      %{
        query:
          K8s.Client.watch("rathole.mrchypark.github.io/v1alpha1", "clients",
            namespace: watching_namespace
          ),
        controller: Rathole.Controller.ClientController
      },
      %{
        query:
          K8s.Client.watch("rathole.mrchypark.github.io/v1alpha1", "servers",
            namespace: watching_namespace
          ),
        controller: Rathole.Controller.ServerController
      }
    ]
  end

  @impl Bonny.Operator
  def crds() do
    [
      %Bonny.API.CRD{
        names: %{
          kind: "Client",
          plural: "clients",
          shortNames: ["cl", "clt"],
          singular: "client"
        },
        group: "rathole.mrchypark.github.io",
        versions: [:"Elixir.Rathole.API.V1Alpha1.Client"],
        scope: :Namespaced
      },
      %Bonny.API.CRD{
        names: %{
          kind: "Server",
          plural: "servers",
          shortNames: ["sv", "svr"],
          singular: "server"
        },
        group: "rathole.mrchypark.github.io",
        versions: [:"Elixir.Rathole.API.V1Alpha1.Server"],
        scope: :Namespaced
      }
    ]
  end
end
