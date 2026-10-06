// Plain, dependency-free client-side facet filtering. Reads the data-*
// attributes Astro already rendered server-side onto each .component-row —
// no fetch, no second data source, no framework. See specs/03-website.md.
(function () {
  const facetGroups = document.querySelectorAll("fieldset[data-facet]");
  const rows = document.querySelectorAll(".component-row");

  function selectedValues(fieldset) {
    return Array.from(fieldset.querySelectorAll("input[type=checkbox]:checked")).map(
      (input) => input.value,
    );
  }

  function rowMatchesFacet(row, facetName, selected) {
    if (selected.length === 0) return true;
    const attr = row.dataset[facetAttrKey(facetName)] || "";
    const rowValues = attr.split(" ").filter(Boolean);
    return selected.some((value) => rowValues.includes(value));
  }

  function facetAttrKey(facetName) {
    // data-category -> dataset.category; data-genres -> dataset.genres, etc.
    const map = {
      category: "category",
      genre: "genres",
      engine: "engines",
      license: "licenses",
      maturity: "maturities",
    };
    return map[facetName] || facetName;
  }

  function applyFilters() {
    const selections = Array.from(facetGroups).map((fieldset) => ({
      name: fieldset.dataset.facet,
      values: selectedValues(fieldset),
    }));

    rows.forEach((row) => {
      const visible = selections.every(({ name, values }) =>
        rowMatchesFacet(row, name, values),
      );
      row.hidden = !visible;
    });
  }

  facetGroups.forEach((fieldset) => {
    fieldset.addEventListener("change", applyFilters);
  });
})();
