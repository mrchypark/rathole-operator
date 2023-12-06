defmodule Rathole.Controller.ServerControllerTest do
  @moduledoc false
  use ExUnit.Case, async: false
  use Bonny.Axn.Test

  alias Rathole.Controller.ServerController

  test "add is handled and returns axn" do
    axn = axn(:add)
    result = ServerController.call(axn, [])
    assert is_struct(result, Bonny.Axn)
  end

  test "modify is handled and returns axn" do
    axn = axn(:modify)
    result = ServerController.call(axn, [])
    assert is_struct(result, Bonny.Axn)
  end

  test "reconcile is handled and returns axn" do
    axn = axn(:reconcile)
    result = ServerController.call(axn, [])
    assert is_struct(result, Bonny.Axn)
  end

  test "delete is handled and returns axn" do
    axn = axn(:delete)
    result = ServerController.call(axn, [])
    assert is_struct(result, Bonny.Axn)
  end
end
