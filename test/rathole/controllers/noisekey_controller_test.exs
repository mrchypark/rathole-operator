defmodule Rathole.Controller.NoiseKeyControllerTest do
  @moduledoc false
  use ExUnit.Case, async: false
  use Bonny.Axn.Test

  alias Rathole.Controller.NoiseKeyController

  test "add is handled and returns axn" do
    axn = axn(:add)
    result = NoiseKeyController.call(axn, [])
    assert is_struct(result, Bonny.Axn)
  end

  test "modify is handled and returns axn" do
    axn = axn(:modify)
    result = NoiseKeyController.call(axn, [])
    assert is_struct(result, Bonny.Axn)
  end

  test "reconcile is handled and returns axn" do
    axn = axn(:reconcile)
    result = NoiseKeyController.call(axn, [])
    assert is_struct(result, Bonny.Axn)
  end

  test "delete is handled and returns axn" do
    axn = axn(:delete)
    result = NoiseKeyController.call(axn, [])
    assert is_struct(result, Bonny.Axn)
  end
end
