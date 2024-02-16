<template>
  <div>
    <h2>Addition</h2>
    <input v-model.number="a" type="number" />
    <input v-model.number="b" type="number" />
    <p v-if="result">{{ result }}</p>
    <button @click="calc">Calculate</button>
  </div>
</template>

<script lang="ts">
import { sum } from '../bindings/sum';

export default {
  data() {
    return {
      a: 2,
      b: 2,
      result: 4
    };
  },
  methods: {
    async calc() {
      this.result = await sum({ a: this.a, b: this.b });
    },
  },
};
</script>