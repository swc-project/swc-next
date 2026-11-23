try {
  run();
} catch (error: unknown /* Parameter */) // Catch header
{
  // Catch body
  recover(error);
}
