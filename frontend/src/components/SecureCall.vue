<template>
  <div>
    <h2>Secure Call</h2>
    <button @click="makeSecureCall">Make Secure Call</button>
    <p v-if="message">{{ message }}</p>
  </div>
</template>

<script lang="ts">
export default {
  data() {
    return {
      message: '',
    };
  },
  methods: {
    async makeSecureCall() {
      try {
        const token = localStorage.getItem('jwt');
        const response = await fetch('http://localhost:3030/api/secure', {
          method: 'GET',
          headers: {
            Authorization: token || undefined,
          } as HeadersInit,
        });
        if (!response.ok) throw new Error('Failed to make secure call');
        const data = await response.text(); // Assuming the response is plain text
        this.message = data;
      } catch (error) {
        if (error instanceof Error) {
            this.message = error.message;
        }
      }
    },
  },
};
</script>
