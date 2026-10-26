import {
  computed,
  defineComponent,
  h,
  inject,
  provide,
} from 'vue'

const selectKey = Symbol('compact-select')
const radioKey = Symbol('compact-radio')

function classNames(...values) {
  return values.filter(Boolean).join(' ')
}

const ElButton = defineComponent({
  name: 'ElButton',
  inheritAttrs: false,
  props: {
    type: { type: String, default: 'default' },
    nativeType: { type: String, default: 'button' },
    size: String,
    loading: Boolean,
    disabled: Boolean,
    link: Boolean,
    text: Boolean,
    plain: Boolean,
    icon: [Object, Function],
  },
  setup(props, { attrs, slots }) {
    return () => h('button', {
      ...attrs,
      type: props.nativeType,
      disabled: props.disabled || props.loading,
      'aria-busy': props.loading || undefined,
      class: classNames(
        'el-button ui-button',
        props.type === 'primary' && 'ui-button-primary',
        props.type === 'danger' && 'ui-button-danger',
        props.type === 'warning' && 'ui-button-warning',
        (props.link || props.text) && 'ui-button-ghost',
        props.plain && 'ui-button-outline',
        props.size === 'small' && 'ui-button-small',
        attrs.class,
      ),
    }, [
      props.loading ? h('span', { class: 'ui-spinner', 'aria-hidden': 'true' }) : null,
      !props.loading && props.icon ? h(props.icon, { 'aria-hidden': 'true' }) : null,
      slots.default?.(),
    ])
  },
})

const ElIcon = defineComponent({
  name: 'ElIcon',
  setup(_, { attrs, slots }) {
    return () => h('span', { ...attrs, class: classNames('el-icon ui-icon', attrs.class) }, slots.default?.())
  },
})

const ElInput = defineComponent({
  name: 'ElInput',
  inheritAttrs: false,
  props: {
    modelValue: [String, Number],
    type: { type: String, default: 'text' },
    rows: [String, Number],
    disabled: Boolean,
    readonly: Boolean,
    placeholder: String,
    clearable: Boolean,
  },
  emits: ['update:modelValue', 'input', 'change', 'clear'],
  setup(props, { attrs, emit, slots }) {
    const onInput = (event) => {
      emit('update:modelValue', event.target.value)
      emit('input', event.target.value)
    }
    const onChange = (event) => emit('change', event.target.value)
    return () => h('div', { class: classNames('el-input ui-input-shell', attrs.class) }, [
      slots.prepend ? h('span', { class: 'ui-input-addon' }, slots.prepend()) : null,
      props.type === 'textarea'
        ? h('textarea', { ...attrs, class: 'ui-input', value: props.modelValue ?? '', rows: props.rows, disabled: props.disabled, readonly: props.readonly, placeholder: props.placeholder, onInput, onChange })
        : h('input', { ...attrs, class: 'ui-input', value: props.modelValue ?? '', type: props.type, disabled: props.disabled, readonly: props.readonly, placeholder: props.placeholder, onInput, onChange }),
      props.clearable && props.modelValue
        ? h('button', { type: 'button', class: 'ui-input-clear', 'aria-label': 'Clear', onClick: () => { emit('update:modelValue', ''); emit('clear') } }, '×')
        : null,
      slots.append ? h('span', { class: 'ui-input-addon' }, slots.append()) : null,
    ])
  },
})

const ElInputNumber = defineComponent({
  name: 'ElInputNumber',
  inheritAttrs: false,
  props: {
    modelValue: Number,
    min: Number,
    max: Number,
    step: { type: Number, default: 1 },
    precision: Number,
    disabled: Boolean,
    controls: { type: Boolean, default: true },
  },
  emits: ['update:modelValue', 'change'],
  setup(props, { attrs, emit }) {
    const update = (event) => {
      const value = event.target.value === '' ? undefined : Number(event.target.value)
      emit('update:modelValue', value)
      emit('change', value)
    }
    return () => h('input', {
      ...attrs,
      class: classNames('el-input-number ui-input', attrs.class),
      type: 'number',
      value: props.modelValue ?? '',
      min: props.min,
      max: props.max,
      step: props.step,
      disabled: props.disabled,
      onInput: update,
    })
  },
})

const ElSwitch = defineComponent({
  name: 'ElSwitch',
  inheritAttrs: false,
  props: {
    modelValue: [Boolean, String, Number],
    activeValue: { default: true },
    inactiveValue: { default: false },
    activeText: String,
    inactiveText: String,
    disabled: Boolean,
  },
  emits: ['update:modelValue', 'change'],
  setup(props, { attrs, emit }) {
    const checked = computed(() => Object.is(props.modelValue, props.activeValue))
    const update = (event) => {
      const value = event.target.checked ? props.activeValue : props.inactiveValue
      emit('update:modelValue', value)
      emit('change', value)
    }
    return () => h('label', { class: classNames('el-switch ui-switch', attrs.class, props.disabled && 'is-disabled') }, [
      props.inactiveText ? h('span', { class: 'ui-switch-label' }, props.inactiveText) : null,
      h('input', { ...attrs, type: 'checkbox', checked: checked.value, disabled: props.disabled, onChange: update }),
      h('span', { class: 'ui-switch-track', 'aria-hidden': 'true' }, h('span', { class: 'ui-switch-thumb' })),
      props.activeText ? h('span', { class: 'ui-switch-label' }, props.activeText) : null,
    ])
  },
})

const ElCheckbox = defineComponent({
  name: 'ElCheckbox',
  inheritAttrs: false,
  props: {
    modelValue: [Boolean, Array],
    value: null,
    label: null,
    disabled: Boolean,
  },
  emits: ['update:modelValue', 'change'],
  setup(props, { attrs, slots, emit }) {
    const optionValue = computed(() => props.value ?? props.label)
    const checked = computed(() => Array.isArray(props.modelValue)
      ? props.modelValue.some((value) => Object.is(value, optionValue.value))
      : Boolean(props.modelValue))
    const update = (event) => {
      let value = event.target.checked
      if (Array.isArray(props.modelValue)) {
        value = [...props.modelValue]
        const index = value.findIndex((item) => Object.is(item, optionValue.value))
        if (event.target.checked && index < 0) value.push(optionValue.value)
        if (!event.target.checked && index >= 0) value.splice(index, 1)
      }
      emit('update:modelValue', value)
      emit('change', value)
    }
    return () => h('label', { class: classNames('el-checkbox ui-check', attrs.class) }, [
      h('input', { ...attrs, type: 'checkbox', checked: checked.value, disabled: props.disabled, onChange: update }),
      h('span', { class: 'ui-check-box', 'aria-hidden': 'true' }),
      h('span', slots.default?.() || String(props.label ?? '')),
    ])
  },
})

const ElSelect = defineComponent({
  name: 'ElSelect',
  inheritAttrs: false,
  props: {
    modelValue: null,
    disabled: Boolean,
    placeholder: String,
    clearable: Boolean,
  },
  emits: ['update:modelValue', 'change'],
  setup(props, { attrs, slots, emit }) {
    const context = {
      modelValue: computed(() => props.modelValue),
    }
    provide(selectKey, context)
    const update = (event) => {
      const option = event.target.selectedOptions[0]
      const value = option?._value ?? option?.value ?? ''
      emit('update:modelValue', value)
      emit('change', value)
    }
    return () => h('select', {
      ...attrs,
      class: classNames('el-select ui-select', attrs.class),
      disabled: props.disabled,
      onChange: update,
    }, [
      props.placeholder ? h('option', { value: '', disabled: !props.clearable, selected: props.modelValue == null || props.modelValue === '' }, props.placeholder) : null,
      slots.default?.(),
    ])
  },
})

const ElOption = defineComponent({
  name: 'ElOption',
  props: { value: null, label: [String, Number], disabled: Boolean },
  setup(props, { slots }) {
    const select = inject(selectKey, null)
    return () => h('option', {
      value: props.value,
      disabled: props.disabled,
      selected: select ? Object.is(select.modelValue.value, props.value) : undefined,
    }, slots.default?.() || String(props.label ?? props.value ?? ''))
  },
})

const ElForm = defineComponent({
  name: 'ElForm',
  setup(_, { attrs, slots, expose }) {
    expose({ validate: async () => true, resetFields: () => {}, clearValidate: () => {} })
    return () => h('form', { ...attrs, class: classNames('el-form ui-form', attrs.class), onSubmit: (event) => event.preventDefault() }, slots.default?.())
  },
})

const ElFormItem = defineComponent({
  name: 'ElFormItem',
  props: { label: String, prop: [String, Array], required: Boolean },
  setup(props, { attrs, slots }) {
    return () => h('label', { ...attrs, class: classNames('el-form-item ui-form-item', attrs.class) }, [
      (props.label || slots.label) ? h('span', { class: 'el-form-item__label ui-form-label' }, slots.label?.() || props.label) : null,
      h('span', { class: 'el-form-item__content ui-form-control' }, slots.default?.()),
    ])
  },
})

const ElTag = defineComponent({
  name: 'ElTag',
  props: { type: String, size: String, closable: Boolean, effect: String },
  emits: ['close'],
  setup(props, { attrs, slots, emit }) {
    return () => h('span', { ...attrs, class: classNames('el-tag ui-tag', props.type && `ui-tag-${props.type}`, attrs.class) }, [
      slots.default?.(),
      props.closable ? h('button', { type: 'button', 'aria-label': 'Remove', onClick: () => emit('close') }, '×') : null,
    ])
  },
})

const ElAlert = defineComponent({
  name: 'ElAlert',
  props: { title: String, description: String, type: { type: String, default: 'info' }, closable: { type: Boolean, default: true }, showIcon: Boolean },
  setup(props, { attrs, slots }) {
    return () => h('section', { ...attrs, class: classNames('el-alert ui-alert', `ui-alert-${props.type}`, attrs.class), role: props.type === 'error' ? 'alert' : 'status' }, [
      props.title ? h('strong', { class: 'el-alert__title' }, props.title) : null,
      props.description ? h('p', { class: 'el-alert__description' }, props.description) : null,
      slots.default?.(),
    ])
  },
})

const ElSlider = defineComponent({
  name: 'ElSlider',
  inheritAttrs: false,
  props: { modelValue: Number, min: Number, max: Number, step: Number, disabled: Boolean, vertical: Boolean, height: String },
  emits: ['update:modelValue', 'change'],
  setup(props, { attrs, emit }) {
    const update = (event) => {
      const value = Number(event.target.value)
      emit('update:modelValue', value)
      emit('change', value)
    }
    return () => h('input', {
      ...attrs,
      class: classNames('el-slider ui-slider', props.vertical && 'is-vertical', attrs.class),
      style: props.vertical && props.height ? { height: props.height } : undefined,
      type: 'range', value: props.modelValue, min: props.min, max: props.max, step: props.step, disabled: props.disabled,
      onInput: update,
    })
  },
})

const ElRadioGroup = defineComponent({
  name: 'ElRadioGroup',
  props: { modelValue: null, disabled: Boolean },
  emits: ['update:modelValue', 'change'],
  setup(props, { attrs, slots, emit }) {
    provide(radioKey, {
      modelValue: computed(() => props.modelValue),
      disabled: computed(() => props.disabled),
      update(value) { emit('update:modelValue', value); emit('change', value) },
    })
    return () => h('div', { ...attrs, class: classNames('el-radio-group ui-radio-group', attrs.class), role: 'radiogroup' }, slots.default?.())
  },
})

const ElRadioButton = defineComponent({
  name: 'ElRadioButton',
  props: { value: null, label: null, disabled: Boolean },
  setup(props, { attrs, slots }) {
    const group = inject(radioKey)
    const value = computed(() => props.value ?? props.label)
    return () => h('label', { ...attrs, class: classNames('el-radio-button ui-radio-button', attrs.class) }, [
      h('input', { type: 'radio', checked: Object.is(group?.modelValue.value, value.value), disabled: props.disabled || group?.disabled.value, onChange: () => group?.update(value.value) }),
      h('span', slots.default?.() || String(value.value ?? '')),
    ])
  },
})

const ElCollapse = defineComponent({
  name: 'ElCollapse',
  setup(_, { attrs, slots }) {
    return () => h('div', { ...attrs, class: classNames('el-collapse ui-collapse', attrs.class) }, slots.default?.())
  },
})

const ElCollapseItem = defineComponent({
  name: 'ElCollapseItem',
  props: { title: String, name: [String, Number] },
  setup(props, { attrs, slots }) {
    return () => h('details', { ...attrs, class: classNames('el-collapse-item ui-collapse-item', attrs.class) }, [
      h('summary', { class: 'el-collapse-item__header' }, slots.title?.() || props.title),
      h('div', { class: 'el-collapse-item__content' }, slots.default?.()),
    ])
  },
})

const ElDescriptions = defineComponent({
  name: 'ElDescriptions',
  setup(_, { attrs, slots }) {
    return () => h('dl', { ...attrs, class: classNames('el-descriptions ui-descriptions', attrs.class) }, slots.default?.())
  },
})

const ElDescriptionsItem = defineComponent({
  name: 'ElDescriptionsItem',
  props: { label: String, span: [String, Number] },
  setup(props, { attrs, slots }) {
    return () => h('div', { ...attrs, class: classNames('el-descriptions-item ui-description', attrs.class), style: props.span ? { gridColumn: `span ${props.span}` } : undefined }, [
      h('dt', { class: 'el-descriptions-item__label' }, slots.label?.() || props.label),
      h('dd', { class: 'el-descriptions-item__content' }, slots.default?.()),
    ])
  },
})

const ElUpload = defineComponent({
  name: 'ElUpload',
  inheritAttrs: false,
  props: { accept: String, disabled: Boolean, onChange: Function, limit: Number, onExceed: Function },
  setup(props, { attrs, slots }) {
    const change = (event) => {
      const files = [...(event.target.files || [])]
      if (props.limit && files.length > props.limit) return props.onExceed?.(files)
      if (files[0]) props.onChange?.({ name: files[0].name, size: files[0].size, raw: files[0] }, files)
      event.target.value = ''
    }
    return () => h('label', { ...attrs, class: classNames('el-upload ui-upload', attrs.class) }, [
      slots.default?.(),
      h('input', { type: 'file', accept: props.accept, disabled: props.disabled, onChange: change }),
    ])
  },
})

const ElProgress = defineComponent({
  name: 'ElProgress',
  props: { percentage: { type: Number, default: 0 }, showText: { type: Boolean, default: true } },
  setup(props, { attrs }) {
    return () => h('div', { ...attrs, class: classNames('el-progress ui-progress', attrs.class), role: 'progressbar', 'aria-valuenow': props.percentage, 'aria-valuemin': 0, 'aria-valuemax': 100 }, [
      h('span', { style: { width: `${Math.min(100, Math.max(0, props.percentage))}%` } }),
      props.showText ? h('em', `${props.percentage}%`) : null,
    ])
  },
})

const ElSkeleton = defineComponent({
  name: 'ElSkeleton',
  props: { rows: { type: Number, default: 3 }, animated: Boolean },
  setup(props, { attrs }) {
    return () => h('div', { ...attrs, class: classNames('el-skeleton ui-skeleton', attrs.class), 'aria-busy': 'true' }, Array.from({ length: props.rows }, (_, index) => h('span', { key: index })))
  },
})

const ElEmpty = defineComponent({
  name: 'ElEmpty',
  props: { description: String },
  setup(props, { attrs, slots }) {
    return () => h('div', { ...attrs, class: classNames('el-empty ui-empty', attrs.class) }, slots.default?.() || props.description || 'No items')
  },
})

const components = {
  ElAlert,
  ElButton,
  ElCheckbox,
  ElCollapse,
  ElCollapseItem,
  ElDescriptions,
  ElDescriptionsItem,
  ElEmpty,
  ElForm,
  ElFormItem,
  ElIcon,
  ElInput,
  ElInputNumber,
  ElOption,
  ElProgress,
  ElRadioButton,
  ElRadioGroup,
  ElSelect,
  ElSkeleton,
  ElSlider,
  ElSwitch,
  ElTag,
  ElUpload,
}

export const compactUi = {
  install(app) {
    for (const [name, component] of Object.entries(components)) app.component(name, component)
    app.directive('loading', {
      mounted(element, binding) { element.toggleAttribute('data-loading', Boolean(binding.value)) },
      updated(element, binding) { element.toggleAttribute('data-loading', Boolean(binding.value)) },
    })
  },
}
