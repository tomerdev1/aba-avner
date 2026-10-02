# Duplicate Review Mock

Rough layout for the pre-export review step.

```text
+---------------------------------------------------------------+
| Review Duplicate Groups                                       |
| Choose one image to keep from each group before export.       |
|                                                               |
| [ Group 1 ]                                                   |
| +---------+  +---------+  +---------+                         |
| | SELECTED|  |         |  |         |                         |
| |  img A  |  |  img B  |  |  img C  |                         |
| +=========+  +---------+  +---------+                         |
|                                                               |
| [ Group 2 ]                                                   |
| +---------+  +---------+                                      |
| |         |  | SELECTED|                                      |
| |  img D  |  |  img E  |                                      |
| +---------+  +=========+                                      |
|                                                               |
| [ Apply & Export ]                           [ Cancel ]       |
+---------------------------------------------------------------+
```

Notes:

- Groups are stacked vertically.
- Each group shows thumbnails in a simple grid.
- Exactly one thumbnail is selected per group.
- The selected image uses a visible border/highlight only.
- No ranking controls, bulk actions, or extra metadata.
