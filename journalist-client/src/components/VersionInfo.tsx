import { EuiLink, EuiText } from "@elastic/eui";

export const VersionInfo = () => {
  const maybeRepo = import.meta.env.VITE_GITHUB_REPO;
  const maybeGithubRepoName = maybeRepo?.startsWith("git@")
    ? maybeRepo.substring(maybeRepo.indexOf(":") + 1, maybeRepo.length - 4) // local repo ssh
    : maybeRepo?.startsWith("https://github.com/")
      ? maybeRepo.substring(
          // https (locally or in GHA)
          19,
          maybeRepo.endsWith(".git") ? maybeRepo.length - 4 : maybeRepo.length,
        )
      : maybeRepo;

  return (
    import.meta.env.VITE_GIT_SHA &&
    maybeGithubRepoName && (
      <EuiText size="xs" textAlign="right" color="grey">
        built from:{" "}
        <EuiLink
          target="_blank"
          href={`https://github.com/${maybeGithubRepoName}/commit/${import.meta.env.VITE_GIT_SHA}`}
          style={{ color: "grey" }}
        >
          {import.meta.env.VITE_GIT_SHA?.substring(0, 7) || "DEV"}
        </EuiLink>
        {import.meta.env.VITE_BUILD_DATE
          ? ` at ${import.meta.env.VITE_BUILD_DATE}`
          : " DEV"}
      </EuiText>
    )
  );
};
