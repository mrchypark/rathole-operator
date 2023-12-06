defmodule Rathole.Controller.ClientControllerTest do
  @moduledoc false
  use ExUnit.Case, async: false
  use Bonny.Axn.Test

  alias Rathole.Controller.ClientController

  test "add is handled and returns axn" do
    axn = axn(:add)
    result = ClientController.call(axn, [])
    assert is_struct(result, Bonny.Axn)
  end

  test "modify is handled and returns axn" do
    axn = axn(:modify)
    result = ClientController.call(axn, [])
    assert is_struct(result, Bonny.Axn)
  end

  test "reconcile is handled and returns axn" do
    axn = axn(:reconcile)
    result = ClientController.call(axn, [])
    assert is_struct(result, Bonny.Axn)
  end

  test "delete is handled and returns axn" do
    axn = axn(:delete)
    result = ClientController.call(axn, [])
    assert is_struct(result, Bonny.Axn)
  end
end
