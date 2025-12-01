/**
 * Commitlint configuration for Conventional Commits
 * See: https://www.conventionalcommits.org/
 */
module.exports = {
  extends: ['@commitlint/config-conventional'],
  rules: {
    // Allowed commit types
    'type-enum': [
      2,
      'always',
      [
        'feat',     // New feature (triggers MINOR version bump)
        'fix',      // Bug fix (triggers PATCH version bump)
        'docs',     // Documentation changes
        'style',    // Code style changes (formatting, whitespace)
        'refactor', // Code refactoring (no feature or fix)
        'perf',     // Performance improvements
        'test',     // Test additions or changes
        'build',    // Build system or dependencies
        'ci',       // CI/CD configuration changes
        'chore',    // Other changes (tooling, config, etc.)
        'revert',   // Revert a previous commit
      ],
    ],
    // Enforce lowercase for subject
    'subject-case': [2, 'never', ['start-case', 'pascal-case', 'upper-case']],
    // Allow empty body
    'body-empty': [0],
    // Allow empty footer
    'footer-empty': [0],
    // Header max length
    'header-max-length': [2, 'always', 100],
  },
};
