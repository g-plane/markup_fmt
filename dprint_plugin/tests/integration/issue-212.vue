<template>
  <u-tooltip
    :text="row.original.acknowledgement.severity.value === notice.severity.value
    ? undefined
    : `The acknowledged severity differs from the notice's, which was ${notice.severity.label}.`"
  >
    <u-badge
      :label="row.original.acknowledgement.severity.label"
      :color="row.original.acknowledgement.severity.color"
    />
  </u-tooltip>
  <div
    :data-raw="cond
          ? `line one
line two keeps raw template literal content`
          : someOtherValueThatIsLongEnoughToKeepTheExpressionBrokenAcrossLines"
  >
  </div>
</template>
