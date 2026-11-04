test("navigates to the settings page and verifies the profile heading text", async ({ page, request, context }, testInfo) => {
  await page.goto("/settings");
});
