defmodule RatholeTest do
  use ExUnit.Case
  doctest Rathole

  test "greets the world" do
    assert Rathole.hello() == :world
  end
end
