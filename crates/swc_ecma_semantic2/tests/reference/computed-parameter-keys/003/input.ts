interface key {}
namespace N {
  const key = "p";
  type F = ({ [key]: v }: { p: number }) => typeof v;
}
