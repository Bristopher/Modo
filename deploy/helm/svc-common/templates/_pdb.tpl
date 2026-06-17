{{/*
SPDX-License-Identifier: AGPL-3.0-or-later
*/}}

{{- define "svc-common.pdb" -}}
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: {{ .Values.svc.name }}-shard
  namespace: {{ .Values.global.namespace }}
spec:
  minAvailable: {{ .Values.pdb.minAvailable | quote }}
  selector:
    matchLabels:
      app: {{ .Values.svc.name }}-shard
{{- end -}}

{{- define "svc-common.router-pdb" -}}
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: {{ .Values.svc.name }}
  namespace: {{ .Values.global.namespace }}
spec:
  minAvailable: {{ .Values.pdb.minAvailable | quote }}
  selector:
    matchLabels:
      app: {{ .Values.svc.name }}
{{- end -}}
