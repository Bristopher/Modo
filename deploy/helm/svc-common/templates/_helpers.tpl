{{/*
SPDX-License-Identifier: AGPL-3.0-or-later
*/}}

{{/*
Standard labels for all resources.
*/}}
{{- define "svc-common.labels" -}}
app.kubernetes.io/name: {{ .Values.svc.name }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/version: {{ .Values.svc.tag | default .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end -}}

{{/*
Selector labels (subset of standard labels).
*/}}
{{- define "svc-common.selectorLabels" -}}
app.kubernetes.io/name: {{ .Values.svc.name }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end -}}

{{/*
Construct full image path from registry + image name + tag.
*/}}
{{- define "svc-common.image" -}}
{{- $registry := required "global.registry is required" .Values.global.registry -}}
{{- $tag := required (printf "svc.tag is required (image: %s)" .Values.svc.image) .Values.svc.tag -}}
{{ $registry }}/{{ .Values.svc.image }}:{{ $tag }}
{{- end -}}
