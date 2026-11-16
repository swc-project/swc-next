test("navigates to the settings page and verifies the profile heading text", async ([page, request, context]) => {
  await page.goto("/settings");
});
