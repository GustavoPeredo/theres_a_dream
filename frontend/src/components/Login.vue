<script lang="ts">
export default {
  data() {
    return {
      userId: '',
    };
  },
  methods: {
    async login() {
      try {
        const response = await fetch('http://localhost:3030/api/login', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            user_id: this.userId,
          }),
        });
        if (!response.ok) throw new Error('Login failed');
        const data = await response.json();
        localStorage.setItem('jwt', data.token);
        alert('Login successful!');
      } catch (error) {
        if (error instanceof Error) {
            alert(error.message);
        }
      }
    },
  },
};
</script>


<template>
  <div>
    <h2>Login</h2>
    <input v-model="userId" placeholder="User ID" />
    <button @click="login">Login</button>
  </div>
</template>