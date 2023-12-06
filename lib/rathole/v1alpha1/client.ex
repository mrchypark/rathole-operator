defmodule Rathole.API.V1Alpha1.Client do
  @moduledoc """
  Rathole: clients CRD V1Alpha1 version.

  Modify the `manifest/0` function in order to override the defaults,
  e.g. to define an openAPIV3 schema, add subresources or additional
  printer columns:

  ```
  def manifest() do
    struct!(
      defaults(),
      name: "v1alpha1",
      schema: %{
        openAPIV3Schema: %{
          type: :object,
          properties: %{
            spec: %{
              type: :object,
              properties: %{
                foos: %{type: :integer}
              }
            },
            status: %{
              ...
            }
          }
        }
      },
      additionalPrinterColumns: [
        %{name: "foos", type: :integer, description: "Number of foos", jsonPath: ".spec.foos"}
      ],
      subresources: %{
        status: %{}
      }
    )
  end
  ```
  """
  use Bonny.API.Version,
    hub: true

  @impl true
  def manifest() do
    path = Path.expand(Path.join(File.cwd!(), "/crd/schema/client.yaml"))
    {:ok, value} = YamlElixir.read_from_file(path)
    defaults()
    |> struct!(schema: value)
    |> add_observed_generation_status()
    |> add_conditions()
  end
end
