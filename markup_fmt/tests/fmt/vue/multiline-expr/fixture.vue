<template>
  <u-tooltip
    :text="row.original.acknowledgement.severity.value === notice.severity.value
    ? undefined
    : `The acknowledged severity differs from the notice's, which was ${notice.severity.label}.`"
  >
    <u-badge :label="row.original.acknowledgement.severity.label" />
  </u-tooltip>
</template>
