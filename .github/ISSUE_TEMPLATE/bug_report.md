name: Issue
description: Report an issue with the dots
labels: ["bug"]
type: "Bug"
title: "[BUG] "
body:
    - type: markdown
      attributes:
          value: "**Welcome to submit a new issue!**\n- It takes only 3 steps, so please be patient :)
    - type: checkboxes
      attributes:
          label: "Step 1. Before you submit"
          description: "Hint: The 2nd and 3rd checkbox is **not** forcely required as you may have failed to do so."
          options:
              - label: I've successfully updated to the latest versions.
                required: false # Not required cuz user may have failed to do so
              - label: I've successfully updated the system packages to the latest.
                required: false # Not required cuz user may have failed to do so
              - label: I've ticked the checkboxes without reading their contents
                required: false # Obviously
    - type: textarea
      attributes:
          label: "Step 2. Version info"
          description: "Run `hyprKCS --version` and paste the result below."
          value: "<details><summary>Version info</summary>\n\n```\n<!-- Run `hyprKCS --version` and paste the result here! -->\n```\n\n</details>"
      validations:
          required: true
    - type: textarea
      attributes:
          label: "Step 3. Describe the issue"
          value: "\n<!-- Firsly describe your issue here! -->\n\n<details><summary>Logs</summary>\n\n```\n<!-- Put your log content here!-->\n```\n\n</details>"
      validations:
          required: true