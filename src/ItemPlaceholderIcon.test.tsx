import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import ItemPlaceholderIcon from "./ItemPlaceholderIcon";

describe("ItemPlaceholderIcon", () => {
  it("renders the first letter of the item name", () => {
    render(<ItemPlaceholderIcon name="Academic's Mortarboard" />);
    expect(screen.getByText("A")).toBeInTheDocument();
  });

  it("ignores a leading apostrophe/quote when picking the letter", () => {
    render(<ItemPlaceholderIcon name="'Mortarboard" />);
    expect(screen.getByText("M")).toBeInTheDocument();
  });
});
